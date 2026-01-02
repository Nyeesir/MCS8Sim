const MEMORY_SIZE: usize = (u16::MAX as usize) + 1;
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
    cycle_counter: u64,
    current_instruction_counter: u64

}

impl Cpu{
    pub fn new() -> Self{
        //todo: ogarnąć prawidłową pozycję stack pointera
        Cpu{a_reg:0, flags:0b00000010, b_reg:0, c_reg:0, d_reg:0, e_reg:0, h_reg:0, l_reg:0, stack_pointer:0x0FFF, program_counter:0, memory: [0; MEMORY_SIZE], interrupts_enabled:true, halted:false, cycle_counter:0, current_instruction_counter:0}
    }

    pub fn run(&mut self){
        loop {
            let instruction = self.memory[self.program_counter as usize];
            //FIXME: skontrolowac czy dodawania do pc ma byc tu czy na koncu, w przypadku zmian na pewno bedzie trzeba zmienic operacje czytajace pamiec takie jak lxi
            self.program_counter = self.program_counter.wrapping_add(1);
            match instruction {
                0x00 | 0x10 | 0x20 | 0x30 =>{
                    //NOP
                    self.current_instruction_counter = 4;
                }
                0x01 => {
                    let address = self.read_u16_from_memory();
                    Self::perform_lxi_operation(&mut self.b_reg, &mut self.c_reg, address);
                    self.current_instruction_counter = 10;

                }
                0x40 => {
                    //MOV B,B idk czy zostawic
                    self.current_instruction_counter = 5;
                }
                0x50 => {
                    //MOV D,B
                    self.d_reg = self.b_reg;
                    self.current_instruction_counter = 5;
                }
                0x60 => {
                    //MOV H,B
                    self.h_reg = self.b_reg;
                    self.current_instruction_counter = 5;
                }
                0x70 => {
                    //MOV M,B
                    self.memory[self.get_address_from_m_as_usize()] = self.b_reg;
                    self.current_instruction_counter = 7;
                }
                0x80 => {
                    //ADD B
                    self.perform_u8_addition(self.b_reg);
                    self.current_instruction_counter = 4;
                }
                0x90 => {
                    //SUB B
                    self.perform_u8_subtraction(self.b_reg);
                    self.current_instruction_counter = 4;
                }
                0xa0 => {
                    //ANA B
                    self.perform_and_operation(self.b_reg);
                    self.current_instruction_counter = 4;
                }
                0xb0 => {
                    //ORA B
                    self.perform_or_operation(self.b_reg);
                    self.current_instruction_counter = 4;
                }
                0xc0 => {
                    //RNZ
                    self.current_instruction_counter = self.perform_rnz_operation();
                }
                0xd0 => {
                    //RNC
                    self.current_instruction_counter =self.perform_rnc_operation();
                }
                0xe0 => {
                    //RPO
                    self.current_instruction_counter =self.perform_rpo_operation();
                }
                0xf0 => {
                    //RPE
                    self.current_instruction_counter =self.perform_rpe_operation();
                }
                _ => {
                    panic!("Instruction not implemented yet");
                }
            }
            self.cycle_counter += self.current_instruction_counter;
        }
    }

    fn get_address_from_m(&self) -> u16{
        ((self.h_reg as u16) << 8) | (self.l_reg as u16)
    }

    fn get_address_from_m_as_usize(&self) -> usize{
        self.get_address_from_m() as usize
    }

    fn set_carry_flag(&mut self, value: bool) {
        if value {
            self.flags |= 0b0000_0001;
        } else {
            self.flags &= !0b0000_0001;
        }
    }
    fn get_carry_flag(&self) -> bool{
        self.flags & 0b0000_0001 != 0
    }
    fn set_sign_flag(&mut self, value: bool){
        if value {
            self.flags |= 0b1000_0000;
        } else {
            self.flags &= !0b1000_0000;
        }
    }
    fn set_zero_flag(&mut self, value: bool){
        if value {
            self.flags |= 0b0100_0000;
        } else {
            self.flags &= !0b0100_0000;
        }
    }
    fn get_zero_flag(&self) -> bool{
        self.flags & 0b0100_0000 != 0
    }
    fn set_parity_flag(&mut self, value: bool){
        if value {
            self.flags |= 0b0000_0100;
        } else {
            self.flags &= !0b0000_0100;
        }
    }
    fn get_parity_flag(&self) -> bool {
        self.flags & 0b0000_0100 != 0
    }
    fn set_auxiliary_carry_flag(&mut self, value: bool){
        if value {
            self.flags |= 0b0001_0000;
        } else {
            self.flags &= !0b0001_0000;
        }
    }

    fn check_accumulator_and_set_sign_flag(&mut self){
        if self.a_reg & 0b1000_0000 == 0b1000_0000 {
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

    fn pop_stack(&mut self) -> (u8,u8){
        //TODO: UPEWNIC SIE ZE DZIALA JAK NALEZY I EWENTUALNIE POPRAWIC PRZY POPIE
        let first_value = self.memory[self.stack_pointer as usize];
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let second_value = self.memory[self.stack_pointer as usize];
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        //the second value is more significant than the first value
        (second_value, first_value)
    }

    fn read_u16_from_memory(&mut self) -> u16{
        let lo = self.memory[self.program_counter as usize];
        self.program_counter = self.program_counter.wrapping_add(1);
        let hi = self.memory[self.program_counter as usize];
        self.program_counter = self.program_counter.wrapping_add(1);
        (hi as u16) << 8 | lo as u16
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

     fn perform_and_operation(&mut self, value: u8){
        self.a_reg &= value;
        self.set_carry_flag(false);
        self.check_accumulator_and_set_zero_flag();
        self.check_accumulator_and_set_sign_flag();
        self.check_accumulator_and_set_parity_flag();
    }

     fn perform_or_operation(&mut self, value: u8){
        self.a_reg |= value;
        self.set_carry_flag(false);
        self.check_accumulator_and_set_zero_flag();
        self.check_accumulator_and_set_sign_flag();
        self.check_accumulator_and_set_parity_flag();
    }

     fn perform_rnz_operation(&mut self) -> u64{
        if !self.get_zero_flag(){
            let (more_significant, less_significant) = self.pop_stack();
            self.program_counter = (more_significant as u16) << 8 | (less_significant as u16);
            return 11;
        }
        return 5;
    }

    fn perform_rnc_operation(&mut self) -> u64{
        if !self.get_carry_flag(){
            let (more_significant, less_significant) = self.pop_stack();
            self.program_counter = (more_significant as u16) << 8 | (less_significant as u16);
            return 11;
        }
        return 5;
    }

    fn perform_rpo_operation(&mut self) -> u64{
        if !self.get_parity_flag() {
            let (more_significant, less_significant) = self.pop_stack();
            self.program_counter = (more_significant as u16) << 8 | (less_significant as u16);
            return 11;
        }
        return 5;
    }

    fn perform_rpe_operation(&mut self) -> u64{
        if self.get_parity_flag() {
            let (more_significant, less_significant) = self.pop_stack();
            self.program_counter = (more_significant as u16) << 8 | (less_significant as u16);
            return 11;
        }
        return 5;
    }

    fn perform_lxi_operation(reg_hi: &mut u8, reg_lo: &mut u8, value: u16) {
        *reg_hi = (value >> 8) as u8;
        *reg_lo = value as u8;
    }

}