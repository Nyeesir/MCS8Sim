use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use super::io_handler::{self, OutputEvent};
use super::{Cpu, CpuState, InstructionTrace};

pub enum SimCommand {
    Run,
    Step,
    Stop,
    Reset,
    SetCyclesLimit(Option<u64>),
}

#[derive(Debug, Clone)]
pub enum SimulationEvent {
    Output(OutputEvent),
    InputStatus(bool),
    CyclesPerSecond(u64),
    Halted(bool),
    CpuState(CpuState),
    MemorySnapshot(Vec<u8>),
    Trace(InstructionTrace),
    TraceBatch(Vec<InstructionTrace>),
}

pub struct SimulationController {
    tx: Sender<SimCommand>,
}

const RUN_BATCH_STEPS: usize = 20_000;
const LIMIT_SLEEP_WINDOW_SECS: f64 = 0.05;
const MAX_CYCLES_LIMIT: u64 = 6_000_000;

impl SimulationController {
    pub fn new(
        mut cpu: Cpu,
        input_receiver: Option<Receiver<u8>>,
        event_sender: Sender<SimulationEvent>,
        publish_debug_events: bool,
        cycles_limit: Option<u64>,
    ) -> Self {
        let (tx, rx): (Sender<SimCommand>, Receiver<SimCommand>) = channel();

        thread::spawn(move || {
            let (output_tx, output_rx) = channel::<OutputEvent>();
            let (input_status_tx, input_status_rx) = channel::<bool>();

            io_handler::set_output_sender(Some(output_tx));
            if let Some(receiver) = input_receiver {
                io_handler::set_input_receiver(Some(receiver));
            }
            io_handler::set_input_status_sender(Some(input_status_tx));
            io_handler::init_for_new_sim();
            publish_snapshot(&cpu, &event_sender, publish_debug_events);
            emit(&event_sender, SimulationEvent::Halted(cpu.is_halted()));
            flush_runtime_events(&output_rx, &input_status_rx, &event_sender);

            let mut running = false;
            let mut cycles_limit = cycles_limit.map(|v| v.min(MAX_CYCLES_LIMIT));
            let mut last_halted = cpu.is_halted();
            let mut cycles_since_report: u64 = 0;
            let mut last_report = Instant::now();
            let mut last_state_report = Instant::now();

            loop {
                if running {
                    if io_handler::is_awaiting_input() {
                        let _ = io_handler::poll_input_ready();
                        flush_runtime_events(&output_rx, &input_status_rx, &event_sender);
                        if let Ok(cmd) = rx.try_recv() {
                            match cmd {
                                SimCommand::Run => running = true,
                                SimCommand::Step => {
                                    let mut traces = Vec::new();
                                    step_once(
                                        &mut cpu,
                                        true,
                                        &mut cycles_since_report,
                                        &mut traces,
                                    );
                                    emit_traces(&event_sender, traces);
                                }
                                SimCommand::Stop => {
                                    let _ = io_handler::clear_input_aborted();
                                    running = false;
                                }
                                SimCommand::Reset => {
                                    let _ = io_handler::clear_input_aborted();
                                    reset_cpu(
                                        &mut cpu,
                                        &event_sender,
                                        publish_debug_events,
                                        &mut cycles_since_report,
                                        &mut last_report,
                                    );
                                }
                                SimCommand::SetCyclesLimit(limit) => {
                                    cycles_limit = limit.map(|v| v.min(MAX_CYCLES_LIMIT));
                                }
                            }
                        }
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }

                    if cpu.is_halted() {
                        running = false;
                    } else {
                        let batch_start = Instant::now();
                        let mut steps = 0usize;
                        let mut batch_cycles = 0u64;
                        let mut batch_traces = Vec::new();
                        let max_cycles = cycles_limit
                            .map(|limit| (limit as f64 * LIMIT_SLEEP_WINDOW_SECS).ceil() as u64)
                            .unwrap_or(u64::MAX)
                            .max(1);

                        while steps < RUN_BATCH_STEPS && !cpu.is_halted() && batch_cycles < max_cycles
                        {
                            batch_cycles += step_once(
                                &mut cpu,
                                publish_debug_events,
                                &mut cycles_since_report,
                                &mut batch_traces,
                            );
                            steps += 1;

                            if io_handler::clear_input_aborted() {
                                running = false;
                                break;
                            }

                            flush_runtime_events(&output_rx, &input_status_rx, &event_sender);

                            if let Ok(cmd) = rx.try_recv() {
                                match cmd {
                                    SimCommand::Run => running = true,
                                    SimCommand::Step => {
                                        batch_cycles += step_once(
                                            &mut cpu,
                                            publish_debug_events,
                                            &mut cycles_since_report,
                                            &mut batch_traces,
                                        );
                                    }
                                    SimCommand::Stop => {
                                        let _ = io_handler::clear_input_aborted();
                                        running = false;
                                        break;
                                    }
                                    SimCommand::Reset => {
                                        let _ = io_handler::clear_input_aborted();
                                        reset_cpu(
                                            &mut cpu,
                                            &event_sender,
                                            publish_debug_events,
                                            &mut cycles_since_report,
                                            &mut last_report,
                                        );
                                        batch_cycles = 0;
                                        break;
                                    }
                                    SimCommand::SetCyclesLimit(limit) => {
                                        cycles_limit = limit.map(|v| v.min(MAX_CYCLES_LIMIT));
                                    }
                                }
                            }
                        }

                        if !batch_traces.is_empty() {
                            emit(&event_sender, SimulationEvent::TraceBatch(batch_traces));
                        }

                        if let Some(limit) = cycles_limit {
                            let expected = (batch_cycles as f64) / (limit as f64);
                            let actual = batch_start.elapsed().as_secs_f64();
                            if expected > actual {
                                thread::sleep(Duration::from_secs_f64(expected - actual));
                            }
                        }
                    }

                    publish_halted(&cpu, &event_sender, &mut last_halted);
                    flush_runtime_events(&output_rx, &input_status_rx, &event_sender);

                    if let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            SimCommand::Run => running = true,
                            SimCommand::Step => {
                                let mut traces = Vec::new();
                                step_once(
                                    &mut cpu,
                                    publish_debug_events,
                                    &mut cycles_since_report,
                                    &mut traces,
                                );
                                emit_traces(&event_sender, traces);
                            }
                            SimCommand::Stop => {
                                let _ = io_handler::clear_input_aborted();
                                running = false;
                            }
                            SimCommand::Reset => {
                                let _ = io_handler::clear_input_aborted();
                                reset_cpu(
                                    &mut cpu,
                                    &event_sender,
                                    publish_debug_events,
                                    &mut cycles_since_report,
                                    &mut last_report,
                                );
                            }
                            SimCommand::SetCyclesLimit(limit) => {
                                cycles_limit = limit.map(|v| v.min(MAX_CYCLES_LIMIT));
                            }
                        }
                    }

                    let elapsed = last_report.elapsed();
                    if elapsed >= Duration::from_millis(500) {
                        let cps = (cycles_since_report as f64) / elapsed.as_secs_f64();
                        emit(
                            &event_sender,
                            SimulationEvent::CyclesPerSecond(cps.round() as u64),
                        );
                        cycles_since_report = 0;
                        last_report = Instant::now();
                    }

                    if publish_debug_events
                        && last_state_report.elapsed() >= Duration::from_millis(500)
                    {
                        publish_snapshot(&cpu, &event_sender, publish_debug_events);
                        last_state_report = Instant::now();
                    }

                    flush_runtime_events(&output_rx, &input_status_rx, &event_sender);

                    continue;
                }

                match rx.recv() {
                    Ok(cmd) => match cmd {
                        SimCommand::Run => running = true,
                        SimCommand::Step => {
                            let mut traces = Vec::new();
                            step_once(
                                &mut cpu,
                                publish_debug_events,
                                &mut cycles_since_report,
                                &mut traces,
                            );
                            emit_traces(&event_sender, traces);
                            let input_aborted = io_handler::clear_input_aborted();
                            let elapsed = last_report.elapsed();
                            let cps = if elapsed.as_secs_f64() > 0.0 {
                                (cycles_since_report as f64) / elapsed.as_secs_f64()
                            } else {
                                0.0
                            };
                            emit(
                                &event_sender,
                                SimulationEvent::CyclesPerSecond(cps.round() as u64),
                            );
                            publish_snapshot(&cpu, &event_sender, publish_debug_events);
                            cycles_since_report = 0;
                            last_report = Instant::now();
                            last_state_report = Instant::now();
                            publish_halted(&cpu, &event_sender, &mut last_halted);
                            flush_runtime_events(&output_rx, &input_status_rx, &event_sender);
                            if input_aborted {
                                running = false;
                            }
                        }
                        SimCommand::Stop => {
                            let _ = io_handler::clear_input_aborted();
                            running = false;
                        }
                        SimCommand::Reset => {
                            let _ = io_handler::clear_input_aborted();
                            reset_cpu(
                                &mut cpu,
                                &event_sender,
                                publish_debug_events,
                                &mut cycles_since_report,
                                &mut last_report,
                            );
                            last_state_report = Instant::now();
                            publish_halted(&cpu, &event_sender, &mut last_halted);
                        }
                        SimCommand::SetCyclesLimit(limit) => {
                            cycles_limit = limit.map(|v| v.min(MAX_CYCLES_LIMIT));
                        }
                    },
                    Err(_) => break,
                }
            }
        });
        Self { tx }
    }

    pub fn run(&self) {
        let _ = self.tx.send(SimCommand::Run);
    }

    pub fn step(&self) {
        let _ = self.tx.send(SimCommand::Step);
    }

    pub fn stop(&self) {
        io_handler::abort_input_wait();
        let _ = self.tx.send(SimCommand::Stop);
    }

    pub fn reset(&self) {
        io_handler::reset_io_state();
        let _ = self.tx.send(SimCommand::Stop);
        let _ = self.tx.send(SimCommand::Reset);
    }

    pub fn set_cycles_limit(&self, limit: Option<u64>) {
        let _ = self.tx.send(SimCommand::SetCyclesLimit(limit));
    }
}

fn emit(sender: &Sender<SimulationEvent>, event: SimulationEvent) {
    let _ = sender.send(event);
}

fn emit_traces(sender: &Sender<SimulationEvent>, traces: Vec<InstructionTrace>) {
    if traces.is_empty() {
        return;
    }

    if traces.len() == 1 {
        emit(sender, SimulationEvent::Trace(traces.into_iter().next().unwrap()));
    } else {
        emit(sender, SimulationEvent::TraceBatch(traces));
    }
}

fn flush_runtime_events(
    output_rx: &Receiver<OutputEvent>,
    input_status_rx: &Receiver<bool>,
    event_sender: &Sender<SimulationEvent>,
) {
    for output in output_rx.try_iter() {
        emit(event_sender, SimulationEvent::Output(output));
    }
    for waiting in input_status_rx.try_iter() {
        emit(event_sender, SimulationEvent::InputStatus(waiting));
    }
}

fn publish_snapshot(cpu: &Cpu, event_sender: &Sender<SimulationEvent>, publish_debug_events: bool) {
    if !publish_debug_events {
        return;
    }

    emit(event_sender, SimulationEvent::CpuState(cpu.snapshot()));
    emit(
        event_sender,
        SimulationEvent::MemorySnapshot(cpu.memory_snapshot()),
    );
}

fn publish_halted(cpu: &Cpu, event_sender: &Sender<SimulationEvent>, last_halted: &mut bool) {
    let halted = cpu.is_halted();
    if halted != *last_halted {
        emit(event_sender, SimulationEvent::Halted(halted));
        *last_halted = halted;
    }
}

fn step_once(
    cpu: &mut Cpu,
    emit_trace: bool,
    cycles_since_report: &mut u64,
    traces: &mut Vec<InstructionTrace>,
) -> u64 {
    if emit_trace {
        let (cycles, trace) = cpu.step_with_trace();
        if !io_handler::take_trace_suppress() {
            traces.push(trace);
        }
        *cycles_since_report += cycles;
        cycles
    } else {
        let cycles = cpu.step_with_cycles();
        *cycles_since_report += cycles;
        cycles
    }
}

fn reset_cpu(
    cpu: &mut Cpu,
    event_sender: &Sender<SimulationEvent>,
    publish_debug_events: bool,
    cycles_since_report: &mut u64,
    last_report: &mut Instant,
) {
    cpu.reset();
    *cycles_since_report = 0;
    *last_report = Instant::now();
    emit(event_sender, SimulationEvent::CyclesPerSecond(0));
    emit(event_sender, SimulationEvent::Halted(cpu.is_halted()));
    publish_snapshot(cpu, event_sender, publish_debug_events);
}
