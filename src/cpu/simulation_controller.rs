use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::{Duration, Instant};
use super::{Cpu, CpuState, InstructionTrace, io_handler};

pub enum SimCommand {
    Run,
    Step,
    Stop,
    Reset,
    SetCyclesLimit(Option<u64>),
}

pub struct SimulationController {
    tx: Sender<SimCommand>,
}

const RUN_BATCH_STEPS: usize = 1000;
const LIMIT_SLEEP_WINDOW_SECS: f64 = 0.05;
const MAX_CYCLES_LIMIT: u64 = 6_000_000;

impl SimulationController {
    pub fn new(
        mut cpu: Cpu,
        output_sender: Option<Sender<String>>,
        input_receiver: Option<Receiver<u8>>,
        input_status_sender: Option<Sender<bool>>,
        cycles_sender: Option<Sender<u64>>,
        halted_sender: Option<Sender<bool>>,
        state_sender: Option<Sender<CpuState>>,
        memory_sender: Option<Sender<Vec<u8>>>,
        trace_sender: Option<Sender<InstructionTrace>>,
        cycles_limit: Option<u64>,
    ) -> Self {
        let (tx, rx): (Sender<SimCommand>, Receiver<SimCommand>) = channel();

        thread::spawn(move || {
            if let Some(sender) = output_sender {
                io_handler::set_output_sender(Some(sender));
            }
            if let Some(receiver) = input_receiver {
                io_handler::set_input_receiver(Some(receiver));
            }
            if let Some(sender) = input_status_sender {
                io_handler::set_input_status_sender(Some(sender));
            }
            if let Some(sender) = state_sender.as_ref() {
                let _ = sender.send(cpu.snapshot());
            }
            if let Some(sender) = memory_sender.as_ref() {
                let _ = sender.send(cpu.memory_snapshot());
            }

            let mut running = false;
            let mut cycles_limit = cycles_limit.map(|v| v.min(MAX_CYCLES_LIMIT));
            let mut last_halted = cpu.is_halted();
            let mut cycles_since_report: u64 = 0;
            let mut last_report = Instant::now();
            let mut last_state_report = Instant::now();

            loop {
                if running {
                    if cpu.is_halted() {
                        running = false;
                    } else {
                        let batch_start = Instant::now();
                        let mut steps = 0usize;
                        let mut batch_cycles = 0u64;
                        let max_cycles = cycles_limit
                            .map(|limit| (limit as f64 * LIMIT_SLEEP_WINDOW_SECS).ceil() as u64)
                            .unwrap_or(u64::MAX)
                            .max(1);

                        while steps < RUN_BATCH_STEPS && !cpu.is_halted() && batch_cycles < max_cycles {
                            if let Some(sender) = trace_sender.as_ref() {
                                let (cycles, trace) = cpu.step_with_trace();
                                let _ = sender.send(trace);
                                batch_cycles += cycles;
                            } else {
                                batch_cycles += cpu.step_with_cycles();
                            }
                            steps += 1;
                        }
                        cycles_since_report += batch_cycles;
                        if let Some(limit) = cycles_limit {
                            let expected = (batch_cycles as f64) / (limit as f64);
                            let actual = batch_start.elapsed().as_secs_f64();
                            if expected > actual {
                                thread::sleep(Duration::from_secs_f64(expected - actual));
                            }
                        }
                    }
                    let halted = cpu.is_halted();
                    if halted != last_halted {
                        if let Some(sender) = halted_sender.as_ref() {
                            let _ = sender.send(halted);
                        }
                        last_halted = halted;
                    }

                    if let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            SimCommand::Run => running = true,
                            SimCommand::Step => {
                                if let Some(sender) = trace_sender.as_ref() {
                                    let (cycles, trace) = cpu.step_with_trace();
                                    let _ = sender.send(trace);
                                    cycles_since_report += cycles;
                                } else {
                                    cycles_since_report += cpu.step_with_cycles();
                                }
                            }
                            SimCommand::Stop => running = false,
                            SimCommand::Reset => {
                                cpu.reset();
                                cycles_since_report = 0;
                                last_report = Instant::now();
                                if let Some(sender) = cycles_sender.as_ref() {
                                    let _ = sender.send(0);
                                }
                                if let Some(sender) = halted_sender.as_ref() {
                                    let _ = sender.send(cpu.is_halted());
                                }
                                if let Some(sender) = state_sender.as_ref() {
                                    let _ = sender.send(cpu.snapshot());
                                }
                                if let Some(sender) = memory_sender.as_ref() {
                                    let _ = sender.send(cpu.memory_snapshot());
                                }
                            }
                            SimCommand::SetCyclesLimit(limit) => {
                                cycles_limit = limit.map(|v| v.min(MAX_CYCLES_LIMIT));
                            }
                        }
                    }

                    let elapsed = last_report.elapsed();
                    if elapsed >= Duration::from_millis(500) {
                        if let Some(sender) = cycles_sender.as_ref() {
                            let cps = (cycles_since_report as f64) / elapsed.as_secs_f64();
                            let _ = sender.send(cps.round() as u64);
                        }
                        cycles_since_report = 0;
                        last_report = Instant::now();
                    }

                    if last_state_report.elapsed() >= Duration::from_millis(100) {
                        if let Some(sender) = state_sender.as_ref() {
                            let _ = sender.send(cpu.snapshot());
                        }
                        if let Some(sender) = memory_sender.as_ref() {
                            let _ = sender.send(cpu.memory_snapshot());
                        }
                        last_state_report = Instant::now();
                    }

                    if cycles_limit.is_none() {
                        thread::sleep(Duration::from_millis(1));
                    }
                    continue;
                }

                match rx.recv() {
                    Ok(cmd) => match cmd {
                        SimCommand::Run => running = true,
                        SimCommand::Step => {
                            if let Some(sender) = trace_sender.as_ref() {
                                let (cycles, trace) = cpu.step_with_trace();
                                let _ = sender.send(trace);
                                cycles_since_report += cycles;
                            } else {
                                cycles_since_report += cpu.step_with_cycles();
                            }
                            if let Some(sender) = cycles_sender.as_ref() {
                                let elapsed = last_report.elapsed();
                                let cps = if elapsed.as_secs_f64() > 0.0 {
                                    (cycles_since_report as f64) / elapsed.as_secs_f64()
                                } else {
                                    0.0
                                };
                                let _ = sender.send(cps.round() as u64);
                            }
                            if let Some(sender) = state_sender.as_ref() {
                                let _ = sender.send(cpu.snapshot());
                            }
                            if let Some(sender) = memory_sender.as_ref() {
                                let _ = sender.send(cpu.memory_snapshot());
                            }
                            cycles_since_report = 0;
                            last_report = Instant::now();
                            last_state_report = Instant::now();
                            let halted = cpu.is_halted();
                            if halted != last_halted {
                                if let Some(sender) = halted_sender.as_ref() {
                                    let _ = sender.send(halted);
                                }
                                last_halted = halted;
                            }
                        }
                        SimCommand::Stop => running = false,
                        SimCommand::Reset => {
                            cpu.reset();
                            cycles_since_report = 0;
                            last_report = Instant::now();
                            last_state_report = Instant::now();
                            if let Some(sender) = cycles_sender.as_ref() {
                                let _ = sender.send(0);
                            }
                            if let Some(sender) = halted_sender.as_ref() {
                                let _ = sender.send(cpu.is_halted());
                            }
                            if let Some(sender) = state_sender.as_ref() {
                                let _ = sender.send(cpu.snapshot());
                            }
                            if let Some(sender) = memory_sender.as_ref() {
                                let _ = sender.send(cpu.memory_snapshot());
                            }
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
        let _ = self.tx.send(SimCommand::Stop);
    }

    pub fn reset(&self) {
        let _ = self.tx.send(SimCommand::Reset);
    }

    pub fn set_cycles_limit(&self, limit: Option<u64>) {
        let _ = self.tx.send(SimCommand::SetCyclesLimit(limit));
    }
}
