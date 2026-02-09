use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::Sender;

thread_local! {
    static OUTPUT_SENDER: RefCell<Option<Sender<String>>> = RefCell::new(None);
}

static PORT0X85: AtomicU8 = AtomicU8::new(1);
static PORT0X84: AtomicU8 = AtomicU8::new(1);

pub fn set_output_sender(sender: Option<Sender<String>>) {
    OUTPUT_SENDER.with(|cell| {
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
        0x85 => {
            // PORT0X85.store(value, Ordering::Relaxed);
        }
        _ => {}
    }
}

pub fn handle_input(device: u8) -> u8 {
    match device {
        0x85 => PORT0X85.load(Ordering::Relaxed),
        0x84 => PORT0X84.load(Ordering::Relaxed),
        _ => 0x01,
    }
}
