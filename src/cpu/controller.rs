use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel};
use super::Cpu;

pub enum SimCommand {
    Run,
    Step,
    Stop,
    Reset,
}

pub (crate) struct SimulatorController {
    tx: Sender<SimCommand>,
}

impl SimulatorController {
    pub fn new(mut cpu: Cpu) -> Self {
        let (tx, rx): (Sender<SimCommand>, Receiver<SimCommand>) = channel();

        thread::spawn(move || {
            let mut running = false;

            loop {
                if let Ok(cmd) = rx.recv() {
                    match cmd {
                        SimCommand::Run => {
                            running = true;
                            while running {
                                cpu.step();

                                // throttling
                                std::thread::sleep(std::time::Duration::from_millis(1));
                            }
                        }
                        SimCommand::Step => cpu.step(),
                        SimCommand::Stop => running = false,
                        SimCommand::Reset => cpu.reset(),
                    }
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