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
        loop {
            let instruction = self.memory[self.program_counter as usize];
            match instruction {
                0x00 | 0x10 | 0x20 | 0x30 =>{
                    //NOP
                    self.program_counter += 1;
                }
                0x40 => {
                    //MOV B,B idk czy zostawic
                    self.program_counter += 1;
                }
                0x50 => {
                    //MOV D,B
                    self.d_reg = self.b_reg;
                    self.program_counter += 1;
                }
                0x60 => {
                    //MOV H,B
                    self.h_reg = self.b_reg;
                    self.program_counter += 1;
                }
                0x70 => {
                    //MOV M,B
                    self.memory[self.get_address_from_m_as_usize()] = self.b_reg;
                    self.program_counter += 1;
                }
                0x80 => {
                    //ADD B
                    self.perform_u8_addition(self.b_reg);

                    self.program_counter += 1;
                }
                0x90 => {
                    //SUB B
                    self.perform_u8_subtraction(self.b_reg);
                    self.program_counter += 1;
                }
                0xa0 => {
                    //ANA B
                    self.perform_and_operation(self.b_reg);
                    self.program_counter += 1;
                }
                0xb0 => {
                    //ORA B
                    self.perform_or_operation(self.b_reg);
                    self.program_counter += 1;
                }
                _ => {
                    panic!("Instruction not implemented yet");
                }
            }
        }
    }

    fn get_address_from_m(&self) -> u16{
        (self.h_reg as u16) << 8 & (self.l_reg as u16) << 16
    }

    fn get_address_from_m_as_usize(&self) -> usize{
        self.get_address_from_m() as usize
    }

    fn set_carry_flag(&mut self, value: bool){
        self.flags &= value as u8;
    }
    fn set_sign_flag(&mut self, value: bool){
        self.flags &= (value as u8) << 7;
    }
    fn set_zero_flag(&mut self, value: bool){
        self.flags &= (value as u8) << 1;
    }
    fn set_parity_flag(&mut self, value: bool){
        self.flags &= (value as u8) << 2;
    }
    fn set_auxiliary_carry_flag(&mut self, value: bool){
        self.flags &= (value as u8) << 4;
    }

    fn check_accumulator_and_set_sign_flag(&mut self){
        if self.a_reg & 0b10000000 == 0b10000000 {
            self.set_sign_flag(true)
        }
        else {
            self.set_sign_flag(false)
        }
    }

    fn check_accumulator_and_set_zero_flag(&mut self){
        if self.a_reg == 0 {
            self.set_zero_flag(true)
        }
        else {
            self.set_zero_flag(false)
        }
    }

    fn check_accumulator_and_set_parity_flag(&mut self){
        let ones = self.a_reg.count_ones();
        if ones % 2 == 0 {
            self.set_parity_flag(true)
        }
        else {
            self.set_parity_flag(false)
        }
    }

    fn perform_u8_addition(&mut self, value: u8){
        let (result, carry) = self.a_reg.overflowing_add(value);
        self.set_carry_flag(carry);

        let aux_carry = ((self.a_reg & 0x0F) + (value & 0x0F)) > 0x0F;
        self.set_auxiliary_carry_flag(aux_carry);

        self.a_reg = result;
        self.check_accumulator_and_set_sign_flag();
        self.check_accumulator_and_set_zero_flag();
        self.check_accumulator_and_set_parity_flag();
    }

    fn perform_u8_subtraction(&mut self, value: u8) {
        //TODO: zastanowic sie czy to jest na pewno poprawne
        let (result, borrow) = self.a_reg.overflowing_sub(value);
        self.set_carry_flag(borrow);

        let aux_carry = (self.a_reg & 0x0F) < (value & 0x0F);
        self.set_auxiliary_carry_flag(aux_carry);

        self.a_reg = result;
        self.check_accumulator_and_set_sign_flag();
        self.check_accumulator_and_set_zero_flag();
        self.check_accumulator_and_set_parity_flag();
    }

    pub fn perform_and_operation(&mut self, value: u8){
        self.a_reg &= value;
        self.set_carry_flag(false);
        self.check_accumulator_and_set_zero_flag();
        self.check_accumulator_and_set_sign_flag();
        self.check_accumulator_and_set_parity_flag();
    }

    pub fn perform_or_operation(&mut self, value: u8){
        self.a_reg |= value;
        self.set_carry_flag(false);
        self.check_accumulator_and_set_zero_flag();
        self.check_accumulator_and_set_sign_flag();
        self.check_accumulator_and_set_parity_flag();
    }



}