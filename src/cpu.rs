const MEMORY_SIZE: usize = u16::MAX as usize + 1;
pub struct Cpu{
    a_reg: u8,
    flags: u8,
    b_reg: u8,
    c_reg: u8,
    d_reg: u8,
    e_reg: u8,
    h_reg: u8,
    l_reg: u8,
    stack_pointer: u16,
    program_counter: u16,
    memory: [u8; MEMORY_SIZE],
    interrupts_enabled: bool,
    halted: bool,
}

impl Cpu{
    pub fn new() -> Self{
        Cpu{a_reg:0, flags:0, b_reg:0, c_reg:0, d_reg:0, e_reg:0, h_reg:0, l_reg:0, stack_pointer:0, program_counter:0, memory: [0; MEMORY_SIZE], interrupts_enabled:true, halted:false}
    }

    pub fn run(&mut self){

    }
}