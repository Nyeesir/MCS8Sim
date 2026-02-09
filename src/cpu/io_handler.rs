use std::cell::RefCell;
use std::convert::Into;
use std::io::{self, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{Receiver, Sender};

//TODO: FIX INPUT SOMEHOW
//IT SHOULD WORK BUT IT DOESNT AS INTENTENDED
//MAYBE WRONG PORT AND 0x85 IS ONLY A READY FEEDBACK FROM SCREEN

thread_local! {
    static OUTPUT_SENDER: RefCell<Option<Sender<String>>> = RefCell::new(None);
    static INPUT_RECEIVER: RefCell<Option<Receiver<u8>>> = RefCell::new(None);
    static INPUT_STATUS_SENDER: RefCell<Option<Sender<bool>>> = RefCell::new(None);
}

static PORT0X85: AtomicU8 = AtomicU8::new(1);
static PORT0X84: AtomicU8 = AtomicU8::new(b'2');
static PORT0XA4: AtomicU8 = AtomicU8::new(b'3');
static PORT0XA0: AtomicU8 = AtomicU8::new(b'4');
static PORT0X88: AtomicU8 = AtomicU8::new(b'5');

pub fn set_output_sender(sender: Option<Sender<String>>) {
    OUTPUT_SENDER.with(|cell| {
        *cell.borrow_mut() = sender;
    });
}

pub fn set_input_receiver(receiver: Option<Receiver<u8>>) {
    INPUT_RECEIVER.with(|cell| {
        *cell.borrow_mut() = receiver;
    });
}

pub fn set_input_status_sender(sender: Option<Sender<bool>>) {
    INPUT_STATUS_SENDER.with(|cell| {
        *cell.borrow_mut() = sender;
    });
}

pub fn handle_output(device: u8, value: u8) {
    match device {
        0x84 => {
            let mut output = String::new();
            match value {
                0x0D => output.push('\r'),
                0x0A => output.push('\n'),
                0x09 => output.push('\t'),
                0x1B => {}
                _ => output.push(value as char),
            }

            if !output.is_empty() {
                let sent = OUTPUT_SENDER.with(|cell| {
                    if let Some(sender) = cell.borrow().as_ref() {
                        sender.send(output.clone()).is_ok()
                    } else {
                        false
                    }
                });

                if !sent {
                    print!("{output}");
                    let _ = io::stdout().flush();
                }
            }

            // PORT0X84.store(value, Ordering::Relaxed);
        }
        // 0x85 => {
            // PORT0X85.store(value, Ordering::Relaxed);
        // }
        _ => {}
    }
}

pub fn handle_input(device: u8) -> u8 {
    match device {
        0x85 => {
            // handle_output(0x84, PORT0X85.load(Ordering::Relaxed));
            PORT0X85.load(Ordering::Relaxed)
        },
        // 0x85 => INPUT_RECEIVER.with(|cell| {
        //     let mut receiver = cell.borrow_mut();
        //     let Some(rx) = receiver.as_mut() else {
        //         return 0x01;
        //     };
        //
        //     INPUT_STATUS_SENDER.with(|status_cell| {
        //         if let Some(sender) = status_cell.borrow().as_ref() {
        //             let _ = sender.send(true);
        //         }
        //     });
        //
        //     let value = rx.recv().unwrap_or(0x01);
        //
        //     INPUT_STATUS_SENDER.with(|status_cell| {
        //         if let Some(sender) = status_cell.borrow().as_ref() {
        //             let _ = sender.send(false);
        //         }
        //     });
        //
        //     PORT0X85.store(value, Ordering::Relaxed);
        //     handle_output(0x84, value);
        //     value
        // }),
        0x84 => {
            handle_output(0x84, PORT0X85.load(Ordering::Relaxed));
            PORT0X84.load(Ordering::Relaxed)
        },
        0xA4 => {
            handle_output(0x84, PORT0XA4.load(Ordering::Relaxed));
            PORT0XA4.load(Ordering::Relaxed) },
        0xA0 => {
            handle_output(0x84, PORT0XA0.load(Ordering::Relaxed));
            PORT0XA0.load(Ordering::Relaxed)
        },
        0x88 => {
            handle_output(0x84, PORT0X88.load(Ordering::Relaxed));
            PORT0X88.load(Ordering::Relaxed)
        },
        _ => 0x01,
    }
}
