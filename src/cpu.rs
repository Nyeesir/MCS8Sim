pub mod io_handler;

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
        while !self.halted {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_opcode();
        let cycles = self.execute(opcode);
        self.cycle_counter += cycles;
    }

    fn fetch_opcode(&mut self) -> u8 {
        let opcode = self.memory[self.program_counter as usize];
        self.program_counter = self.program_counter.wrapping_add(1);
        opcode
    }

    fn execute(&mut self, opcode: u8) -> u64 {
        match opcode {
            0x00 | 0x10 | 0x20 | 0x30 => {
                // NOP
                4
            }
            0x01 => {
                //LXI B,d16
                let value = self.read_u16_from_memory();
                Self::perform_lxi_operation_register_pair(&mut self.b_reg, &mut self.c_reg, value);
                10
            }
            0x02 => {
                //STAX B
                self.memory[((self.b_reg as usize) << 8) | (self.c_reg as usize)] = self.a_reg;
                7
            }
            0x03 => {
                //INX B
            }
            0x04 => {
            //INR B
            self.b_reg = self.b_reg.wrapping_add(1);
                //TODO

            }
            0x11 => {
            //LXI D,d16
                let value = self.read_u16_from_memory();
                Self::perform_lxi_operation_register_pair(&mut self.d_reg, &mut self.e_reg, value);
                10
            }
            0x12 => {
                //STAX D
                self.memory[((self.d_reg as usize) << 8) | (self.e_reg as usize)] = self.a_reg;
                7
            }
            0x13 => {
                //INX D
                self.set_de(self.get_de().wrapping_add(1));
                5
            }
            0x21 => {
                //LXI H,d16
                let value = self.read_u16_from_memory();
                Self::perform_lxi_operation_register_pair(&mut self.h_reg, &mut self.l_reg, value);
                10
            }
            0x22 => {
                //SHLD a16
                let mut address =self.read_u16_from_memory();
                self.memory[address as usize] = self.l_reg;
                address = address.wrapping_add(1);
                self.memory[address as usize] = self.h_reg;
                16
            }
            0x23 => {
                //INX H
                self.set_hl(self.get_hl().wrapping_add(1));
                5
            }
            0x31 => {
                //LXI SP,d16
                let value = self.read_u16_from_memory();
                Self::perform_lxi_operation(&mut self.stack_pointer, value);
                10
            }
            0x32 => {
                //STA a16
                let address = self.read_u16_from_memory();
                self.memory[address as usize] = self.a_reg;
                13
            }
            0x33 => {
                //INX SP
                self.stack_pointer = self.stack_pointer.wrapping_add(1);
                5
            }
            0x40 => {
                // MOV B,B
                5
            }
            0x41 => {
                //MOV B,C
                self.b_reg = self.c_reg;
                5
            }
            0x42 => {
                //MOV B,D
                self.b_reg = self.d_reg;
                5
            }
            0x43 => {
                //MOV B,E
                self.b_reg = self.e_reg;
                5
            }
            0x50 => {
                // MOV D,B
                self.d_reg = self.b_reg;
                5
            }
            0x51 => {
                //MOV D,C
                self.d_reg = self.c_reg;
                5
            }
            0x52 => {
                //MOV D,D
                5
            }
            0x53 => {
                //MOV D,E
                self.d_reg = self.e_reg;
                5
            }
            0x60 => {
                // MOV H,B
                self.h_reg = self.b_reg;
                5
            }
            0x61 => {
                //MOV H,C
                self.h_reg = self.c_reg;
                5
            }
            0x62 => {
                //MOV H,D
                self.h_reg = self.d_reg;
                5
            }
            0x63 => {
                //MOV H,E
                self.h_reg = self.e_reg;
                5
            }
            0x70 => {
                // MOV M,B
                let addr = self.get_address_from_m_as_usize();
                self.memory[addr] = self.b_reg;
                7
            }
            0x71 => {
                //MOV M,C
                let addr = self.get_address_from_m_as_usize();
                self.memory[addr] = self.c_reg;
                7
            }
            0x72 => {
                //MOV M,D
                let addr = self.get_address_from_m_as_usize();
                self.memory[addr] = self.d_reg;
                7
            }
            0x73 => {
                //MOV M,E
                let addr = self.get_address_from_m_as_usize();
                self.memory[addr] = self.e_reg;
                7
            }
            0x76 => {
                // HLT
                self.halted = true;
                7
            }
            0x80 => {
                // ADD B
                self.perform_u8_addition(self.b_reg);
                4
            }
            0x81 => {
                //ADD C
                self.perform_u8_addition(self.c_reg);
                4
            }
            0x82 => {
                //ADD D
                self.perform_u8_addition(self.d_reg);
                4
            }
            0x83 => {
                //ADD E
                self.perform_u8_addition(self.e_reg);
                4
            }
            0x90 => {
                // SUB B
                self.perform_u8_subtraction(self.b_reg);
                4
            }
            0x91 => {
                //SUB C
                self.perform_u8_subtraction(self.c_reg);
                4
            }
            0x92 => {
                //SUB D
                self.perform_u8_subtraction(self.d_reg);
                4
            }
            0x93 => {
                //SUB E
                self.perform_u8_subtraction(self.e_reg);
                4
            }
            0xA0 => {
                // ANA B
                self.perform_and_operation(self.b_reg);
                4
            }
            0xA1 => {
                // ANA C
                self.perform_and_operation(self.c_reg);
                4
            }
            0xA2 => {
                //ANA D
                self.perform_and_operation(self.d_reg);
                4
            }
            0xA3 => {
                //ANA E
                self.perform_and_operation(self.e_reg);
                4
            }
            0xB0 => {
                //ORA B
                self.perform_or_operation(self.b_reg);
                4
            }
            0xB1 => {
                //ORA C
                self.perform_or_operation(self.c_reg);
                4
            }
            0xB2 => {
                //ORA D
                self.perform_or_operation(self.d_reg);
                4
            }
            0xB3 => {
                //ORA E
                self.perform_or_operation(self.e_reg);
                4
            }
            0xC0 => {
                //RNZ
                self.perform_rnz_operation()
            }
            0xC1 => {
                //POP B
                let value = self.pop_stack_u16();
                Self::perform_pop_operation(&mut self.b_reg, &mut self.c_reg, value);
                10
            }
            0xC2 => {
                //JNZ a16
                let address = self.read_u16_from_memory();
                if !self.get_zero_flag() {
                    self.program_counter = address;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(2);
                }
                11
            }
            0xC3 => {
                //JMP a16
                self.program_counter = self.read_u16_from_memory();
                10
            }
            0xD0 => {
                //RNC
                self.perform_rnc_operation()
            }
            0xD1 => {
                //POP D
                let value = self.pop_stack_u16();
                Self::perform_pop_operation(&mut self.d_reg, &mut self.e_reg, value);
                10
            }
            0xD2 => {
                //JNC a16
                let address = self.read_u16_from_memory();
                if !self.get_carry_flag() {
                    self.program_counter = address;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(2);
                }
                11
            }
            0xD3 => {
                //OUT d8
                let device = self.read_u8_from_memory();
                io_handler::handle_output(device, self.a_reg);
                10

            }
            0xE0 => {
                //RPO
                self.perform_rpo_operation()
            }
            0xE1 => {
                //POP H
                let value = self.pop_stack_u16();
                Self::perform_pop_operation(&mut self.h_reg, &mut self.l_reg, value);
                10
            }
            0xE2 => {
                //JPO a16
                let address = self.read_u16_from_memory();
                if !self.get_parity_flag() {
                    self.program_counter = address;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(2);
                }
                11
            }
            0xE3 => {
                //XTHL
                let mut temp = self.memory[self.stack_pointer as usize];
                self.memory[self.stack_pointer as usize] = self.l_reg;
                self.l_reg = temp;
                temp = self.memory[(self.stack_pointer as usize).wrapping_add(1)];
                self.memory[(self.stack_pointer as usize).wrapping_add(1)] = self.h_reg;
                self.h_reg = temp;
                18

            }
            0xF0 => {
                //RPE
                self.perform_rpe_operation()
            }
            0xF1 => {
                //POP PSW
                self.perform_pop_psw();
                10
            }
            0xF2 => {
                //JP a16
                let address = self.read_u16_from_memory();
                self.program_counter = address;
                11
            }
            0xF3 => {
                //DI
                self.interrupts_enabled = false;
                4
            }
            _ => panic!("Unimplemented opcode: {:02X}", opcode),
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

    fn get_bc(&self) -> u16 {
        ((self.b_reg as u16) << 8) | self.c_reg as u16
    }

    fn set_bc(&mut self, value: u16) {
        self.b_reg = (value >> 8) as u8;
        self.c_reg = value as u8;
    }

    fn get_de(&self) -> u16 {
        ((self.d_reg as u16) << 8) | self.e_reg as u16
    }

    fn set_de(&mut self, value: u16) {
        self.d_reg = (value >> 8) as u8;
        self.e_reg = value as u8;
    }

    fn get_hl(&self) -> u16 {
        ((self.h_reg as u16) << 8) | self.l_reg as u16
    }

    fn set_hl(&mut self, value: u16) {
        self.h_reg = (value >> 8) as u8;
        self.l_reg = value as u8;
    }

    fn pop_stack_u16(&mut self) -> u16{
        let lo = self.memory[self.stack_pointer as usize];
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let hi = self.memory[self.stack_pointer as usize];
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        ((hi as u16) << 8 )| lo as u16
    }

    fn read_u16_from_memory(&mut self) -> u16{
        let lo = self.memory[self.program_counter as usize];
        self.program_counter = self.program_counter.wrapping_add(1);
        let hi = self.memory[self.program_counter as usize];
        self.program_counter = self.program_counter.wrapping_add(1);
        (hi as u16) << 8 | lo as u16
    }

    fn read_u8_from_memory(&mut self) -> u8{
        let value = self.memory[self.program_counter as usize];
        self.program_counter = self.program_counter.wrapping_add(1);
        value
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
        self.set_auxiliary_carry_flag(true); //TODO: sprawdzi czy na pewno
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
            self.program_counter = self.pop_stack_u16();
            return 11;
        }
        return 5;
    }

    fn perform_rnc_operation(&mut self) -> u64{
        if !self.get_carry_flag(){
            self.program_counter = self.pop_stack_u16();
            return 11;
        }
        return 5;
    }

    fn perform_rpo_operation(&mut self) -> u64{
        if !self.get_parity_flag() {
            self.program_counter = self.pop_stack_u16();
            return 11;
        }
        return 5;
    }

    fn perform_rpe_operation(&mut self) -> u64{
        if self.get_parity_flag() {
            self.program_counter = self.pop_stack_u16();
            return 11;
        }
        return 5;
    }

    fn perform_lxi_operation_register_pair(reg_hi: &mut u8, reg_lo: &mut u8, value: u16) {
        *reg_hi = (value >> 8) as u8;
        *reg_lo = value as u8;
    }

    fn perform_lxi_operation(reg: &mut u16, value: u16) {
        *reg = value;
    }

    fn perform_pop_operation(reg_hi: &mut u8, reg_lo: &mut u8, value: u16) {
        *reg_lo = value as u8;
        *reg_hi = (value >> 8) as u8;
    }

    fn perform_pop_psw(&mut self) {
        let value = self.pop_stack_u16();
        let flags = (value & 0x00FF) as u8;
        let a = (value >> 8) as u8;
        self.flags = flags & 0b1101_0111;
        self.flags |= 0b0000_0010;
        self.a_reg = a;
    }

}