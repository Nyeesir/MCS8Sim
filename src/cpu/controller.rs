use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;
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

            loop {
                if running {
                    if cpu.is_halted() {
                        running = false;
                    } else {
                        cpu.step();
                    }

                    if let Ok(cmd) = rx.try_recv() {
                        match cmd {
                            SimCommand::Run => running = true,
                            SimCommand::Step => cpu.step(),
                            SimCommand::Stop => running = false,
                            SimCommand::Reset => cpu.reset(),
                        }
                    }

                    thread::sleep(Duration::from_millis(1));
                    continue;
                }

                match rx.recv() {
                    Ok(cmd) => match cmd {
                        SimCommand::Run => running = true,
                        SimCommand::Step => cpu.step(),
                        SimCommand::Stop => running = false,
                        SimCommand::Reset => cpu.reset(),
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
