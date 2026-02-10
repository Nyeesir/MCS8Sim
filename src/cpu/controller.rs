use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::{Duration, Instant};
use super::{Cpu, io_handler};

pub enum SimCommand {
    Run,
    Step,
    Stop,
    Reset,
}

pub struct SimulatorController {
    tx: Sender<SimCommand>,
}

impl SimulatorController {
    pub fn new(
        mut cpu: Cpu,
        output_sender: Option<Sender<String>>,
        input_receiver: Option<Receiver<u8>>,
        input_status_sender: Option<Sender<bool>>,
        cycles_sender: Option<Sender<u64>>,
        halted_sender: Option<Sender<bool>>,
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

            let mut running = false;
            let mut last_halted = cpu.is_halted();
            let mut cycles_since_report: u64 = 0;
            let mut last_report = Instant::now();

            loop {
                if running {
                    if cpu.is_halted() {
                        running = false;
                    } else {
                        cycles_since_report += cpu.step_with_cycles();
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
                                cycles_since_report += cpu.step_with_cycles();
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

                    thread::sleep(Duration::from_millis(1));
                    continue;
                }

                match rx.recv() {
                    Ok(cmd) => match cmd {
                        SimCommand::Run => running = true,
                        SimCommand::Step => {
                            cycles_since_report += cpu.step_with_cycles();
                            if let Some(sender) = cycles_sender.as_ref() {
                                let elapsed = last_report.elapsed();
                                let cps = if elapsed.as_secs_f64() > 0.0 {
                                    (cycles_since_report as f64) / elapsed.as_secs_f64()
                                } else {
                                    0.0
                                };
                                let _ = sender.send(cps.round() as u64);
                            }
                            cycles_since_report = 0;
                            last_report = Instant::now();
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
                            if let Some(sender) = cycles_sender.as_ref() {
                                let _ = sender.send(0);
                            }
                            if let Some(sender) = halted_sender.as_ref() {
                                let _ = sender.send(cpu.is_halted());
                            }
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
}
