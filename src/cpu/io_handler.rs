use std::io::{self, Write};

pub fn handle_output(device: u8, value: u8) {
    match device {
        0x84 => {
            // GŁÓWNY PORT WYJŚCIA ZNAKÓW
            match value {
                0x0D => {
                    // CR
                    print!("\r");
                }
                0x0A => {
                    // LF
                    print!("\n");
                }
                0x09 => {
                    // TAB
                    print!("\t");
                }
                0x1B => {
                    // ESC – ignorujemy (prefiks)
                }
                _ => {
                    // normalny znak ASCII
                    print!("{}", value as char);
                }
            }
            // wymuś flush, bo BIOS drukuje znak po znaku
            io::stdout().flush().unwrap();
        }

        0x85 => {
            // port sterujący
        }

        _ => {
        }
    }
}


pub fn handle_input(device: u8) -> u8{
    // println!("IN {:02X}", device);

    match device {
        0x85 => 0x01,
        _ => 0x01,
    }
}

//DB 84 ISTNIEJE
//DB A4 NIE ISTNIEJE
//DB A0 NIE ISTNIEJE
//DB 88 NIE ISTNIEJE
//D3 84 ISTNIEJE
//D3 A4 NIE ISTNIEJE
//D3 A0 NIE ISTNIEJE
//D3 88 ISTNIEJE