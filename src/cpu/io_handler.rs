use std::cell::RefCell;
use std::convert::Into;
use std::io::{self, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use crate::encoding;

//TODO: FIX INPUT SOMEHOW
//IT SHOULD WORK BUT IT DOESNT AS INTENTENDED
//MAYBE WRONG PORT AND 0x85 IS ONLY A READY FEEDBACK FROM SCREEN

//TODO: MAYBE LIMIT KEY RANGE

const TERM_COLS: usize = 90;
const TERM_ROWS: usize = 40;
const TAB_WIDTH: usize = 4;

#[derive(Debug, Clone)]
pub enum OutputEvent {
    Append(String),
    Redraw(String),
}

thread_local! {
    static OUTPUT_SENDER: RefCell<Option<Sender<OutputEvent>>> = RefCell::new(None);
    static INPUT_RECEIVER: RefCell<Option<Receiver<u8>>> = RefCell::new(None);
    static INPUT_STATUS_SENDER: RefCell<Option<Sender<bool>>> = RefCell::new(None);
    static TERMINAL_STATE: RefCell<TerminalState> = RefCell::new(TerminalState::new());
}

static USART0_STATUS: AtomicU8 = AtomicU8::new(1);
static USART0_DATA: AtomicU8 = AtomicU8::new(b'2');
static PORT0XA4: AtomicU8 = AtomicU8::new(b'3');
static PORT0XA0: AtomicU8 = AtomicU8::new(b'4');
static PORT0X88: AtomicU8 = AtomicU8::new(b'5');

pub fn set_output_sender(sender: Option<Sender<OutputEvent>>) {
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
            let event = TERMINAL_STATE.with(|cell| cell.borrow_mut().process_byte(value));
            if let Some(event) = event {
                let sent = OUTPUT_SENDER.with(|cell| {
                    if let Some(sender) = cell.borrow().as_ref() {
                        sender.send(event.clone()).is_ok()
                    } else {
                        false
                    }
                });

                if !sent {
                    let output = match event {
                        OutputEvent::Append(text) => text,
                        OutputEvent::Redraw(text) => text,
                    };
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
            USART0_STATUS.load(Ordering::Relaxed)
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
            handle_output(0x84, USART0_STATUS.load(Ordering::Relaxed));
            USART0_DATA.load(Ordering::Relaxed)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscapeState {
    None,
    Esc,
    EscYRow,
    EscYCol { row: usize },
}

struct TerminalState {
    buffer: Vec<char>,
    cursor_row: usize,
    cursor_col: usize,
    escape_state: EscapeState,
    screen_mode: bool,
}

impl TerminalState {
    fn new() -> Self {
        Self {
            buffer: vec![' '; TERM_COLS * TERM_ROWS],
            cursor_row: 0,
            cursor_col: 0,
            escape_state: EscapeState::None,
            screen_mode: false,
        }
    }

    fn process_byte(&mut self, value: u8) -> Option<OutputEvent> {
        if self.screen_mode {
            self.process_screen_byte(value)
        } else {
            self.process_append_byte(value)
        }
    }

    fn process_append_byte(&mut self, value: u8) -> Option<OutputEvent> {
        match self.escape_state {
            EscapeState::Esc => {
                self.escape_state = EscapeState::None;
                match value {
                    b'H' | b'J' | b'Y' => {
                        self.screen_mode = true;
                        self.clear_screen();
                        self.cursor_row = 0;
                        self.cursor_col = 0;
                        match value {
                            b'H' => return None,
                            b'J' => {
                                self.clear_to_end();
                                return Some(OutputEvent::Redraw(self.render()));
                            }
                            b'Y' => {
                                self.escape_state = EscapeState::EscYRow;
                                return None;
                            }
                            _ => {}
                        }
                    }
                    _ => return self.process_append_plain(value),
                }
            }
            _ => {}
        }

        if value == 0x1B {
            self.escape_state = EscapeState::Esc;
            return None;
        }

        self.process_append_plain(value)
    }

    fn process_append_plain(&self, value: u8) -> Option<OutputEvent> {
        let mut output = String::new();
        let output_char = encoding::cp1252_decode(value);

        match value {
            0x00 | 0x07 | 0x08 | 0x0E | 0x0F | 0x11 | 0x18 => {}
            0x0D => output.push('\r'),
            0x0A => output.push('\n'),
            0x09 => output.push('\t'),
            0x1B => {}
            _ => output.push(output_char),
        }

        if output.is_empty() {
            None
        } else {
            Some(OutputEvent::Append(output))
        }
    }

    fn process_screen_byte(&mut self, value: u8) -> Option<OutputEvent> {
        match self.escape_state {
            EscapeState::Esc => {
                self.escape_state = EscapeState::None;
                match value {
                    b'H' => {
                        self.cursor_row = 0;
                        self.cursor_col = 0;
                        return None;
                    }
                    b'J' => {
                        self.clear_to_end();
                        return Some(OutputEvent::Redraw(self.render()));
                    }
                    b'Y' => {
                        self.escape_state = EscapeState::EscYRow;
                        return None;
                    }
                    _ => return self.process_screen_plain(value),
                }
            }
            EscapeState::EscYRow => {
                let row = value.saturating_sub(0x20) as usize;
                self.escape_state = EscapeState::EscYCol { row };
                return None;
            }
            EscapeState::EscYCol { row } => {
                let col = value.saturating_sub(0x20) as usize;
                self.cursor_row = row.min(TERM_ROWS.saturating_sub(1));
                self.cursor_col = col.min(TERM_COLS.saturating_sub(1));
                self.escape_state = EscapeState::None;
                return None;
            }
            EscapeState::None => {}
        }

        if value == 0x1B {
            self.escape_state = EscapeState::Esc;
            return None;
        }

        self.process_screen_plain(value)
    }

    fn process_screen_plain(&mut self, value: u8) -> Option<OutputEvent> {
        match value {
            0x00 | 0x07 | 0x0E | 0x0F | 0x11 | 0x18 => None,
            0x08 => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
                None
            }
            0x0D => {
                self.cursor_col = 0;
                None
            }
            0x0A => {
                let scrolled = self.new_line();
                if scrolled {
                    Some(OutputEvent::Redraw(self.render()))
                } else {
                    None
                }
            }
            0x09 => {
                let changed = self.tab();
                if changed {
                    Some(OutputEvent::Redraw(self.render()))
                } else {
                    None
                }
            }
            _ => {
                let ch = encoding::cp1252_decode(value);
                self.put_char(ch);
                Some(OutputEvent::Redraw(self.render()))
            }
        }
    }

    fn put_char(&mut self, ch: char) {
        let idx = self.cursor_row * TERM_COLS + self.cursor_col;
        if idx < self.buffer.len() {
            self.buffer[idx] = ch;
        }
        self.advance_cursor();
    }

    fn advance_cursor(&mut self) {
        self.cursor_col += 1;
        if self.cursor_col >= TERM_COLS {
            self.cursor_col = 0;
            self.new_line();
        }
    }

    fn new_line(&mut self) -> bool {
        self.cursor_row += 1;
        if self.cursor_row >= TERM_ROWS {
            self.cursor_row = TERM_ROWS - 1;
            self.scroll_up(1);
            return true;
        }
        false
    }

    fn tab(&mut self) -> bool {
        let next = ((self.cursor_col / TAB_WIDTH) + 1) * TAB_WIDTH;
        let mut changed = false;
        while self.cursor_col < next && self.cursor_col < TERM_COLS {
            let idx = self.cursor_row * TERM_COLS + self.cursor_col;
            if idx < self.buffer.len() {
                self.buffer[idx] = ' ';
                changed = true;
            }
            self.cursor_col += 1;
        }
        if self.cursor_col >= TERM_COLS {
            self.cursor_col = 0;
            self.new_line();
        }
        changed
    }

    fn clear_screen(&mut self) {
        for cell in &mut self.buffer {
            *cell = ' ';
        }
    }

    fn clear_to_end(&mut self) {
        let mut idx = self.cursor_row * TERM_COLS + self.cursor_col;
        while idx < self.buffer.len() {
            self.buffer[idx] = ' ';
            idx += 1;
        }
    }

    fn scroll_up(&mut self, lines: usize) {
        let row_len = TERM_COLS;
        let total = TERM_COLS * TERM_ROWS;
        let shift = lines.min(TERM_ROWS) * row_len;
        if shift == 0 || shift >= total {
            self.clear_screen();
            return;
        }
        self.buffer.copy_within(shift..total, 0);
        for idx in (total - shift)..total {
            self.buffer[idx] = ' ';
        }
    }

    fn render(&self) -> String {
        let mut output = String::with_capacity((TERM_COLS + 1) * TERM_ROWS);
        for row in 0..TERM_ROWS {
            let start = row * TERM_COLS;
            let end = start + TERM_COLS;
            for ch in &self.buffer[start..end] {
                output.push(*ch);
            }
            if row + 1 < TERM_ROWS {
                output.push('\n');
            }
        }
        output
    }
}
