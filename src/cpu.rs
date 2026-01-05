//KONT OD 0XB9
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

}

impl Cpu{
    pub fn new() -> Self{
        //todo: ogarnąć prawidłową pozycję stack pointera
        Cpu{a_reg:0, flags:0b00000010, b_reg:0, c_reg:0, d_reg:0, e_reg:0, h_reg:0, l_reg:0, stack_pointer:0x0FFF, program_counter:0, memory: [0; MEMORY_SIZE], interrupts_enabled:true, halted:false, cycle_counter:0}
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
            0x00 | 0x10 | 0x20 | 0x30 | 0x08 | 0x18 | 0x28 | 0x38 => {
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
                self.set_bc(self.get_bc().wrapping_add(1));
                5
            }
            0x04 => {
                //INR B
                let old = self.b_reg;
                let result = self.b_reg.wrapping_add(1);
                self.b_reg = result;

                self.check_value_and_set_zero_flag(result);
                self.check_value_and_set_sign_flag(result);
                self.check_value_and_set_parity_flag(result);
                self.set_auxiliary_carry_flag((old & 0x0F) + 1 > 0x0F);
                5
            }
            0x05 => {
                //DCR B
                let old = self.b_reg;
                let result = self.b_reg.wrapping_sub(1);
                self.b_reg = result;

                self.check_value_and_set_zero_flag(result);
                self.check_value_and_set_sign_flag(result);
                self.check_value_and_set_parity_flag(result);
                self.set_auxiliary_carry_flag((old & 0x0F) == 0);
                5
            }
            0x06 => {
                //MVI B,d8
                self.b_reg = self.read_u8_from_memory();
                7
            }
            0x07 => {
                //RLC
                let msb = (self.a_reg & 0b1000_0000) != 0;
                self.a_reg = self.a_reg.rotate_left(1);
                self.set_carry_flag(msb);
                4
            }
            0x09 => {
                //DAD B
                self.perform_dad_operation(self.get_bc());
                10
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
            0x14 => {
                //INR D
                let old = self.d_reg;
                let result = self.d_reg.wrapping_add(1);
                self.d_reg = result;

                self.check_value_and_set_zero_flag(result);
                self.check_value_and_set_sign_flag(result);
                self.check_value_and_set_parity_flag(result);
                self.set_auxiliary_carry_flag((old & 0x0F) + 1 > 0x0F);
                5
            }
            0x15 => {
                //DCR D
                let old = self.d_reg;
                let result = self.d_reg.wrapping_sub(1);
                self.d_reg = result;

                self.check_value_and_set_zero_flag(result);
                self.check_value_and_set_sign_flag(result);
                self.check_value_and_set_parity_flag(result);
                self.set_auxiliary_carry_flag((old & 0x0F) == 0);
                5
            }
            0x16 => {
                //MVI D,d8
                self.d_reg = self.read_u8_from_memory();
                7
            }
            0x17 => {
                //RAL
                let old_cy = self.get_carry_flag();
                let msb = (self.a_reg & 0x80) != 0;
                self.a_reg = (self.a_reg << 1) | (old_cy as u8);
                self.set_carry_flag(msb);
                4
            }
            0x19 => {
                //DAD D
                self.perform_dad_operation(self.get_de());
                10
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
            0x24 => {
                //INR H
                let old = self.h_reg;
                let result = self.h_reg.wrapping_add(1);
                self.h_reg = result;

                self.check_value_and_set_zero_flag(result);
                self.check_value_and_set_sign_flag(result);
                self.check_value_and_set_parity_flag(result);
                self.set_auxiliary_carry_flag((old & 0x0F) + 1 > 0x0F);
                5
            }
            0x25 => {
                //DCR H
                let old = self.h_reg;
                let result = self.h_reg.wrapping_sub(1);
                self.h_reg = result;

                self.check_value_and_set_zero_flag(result);
                self.check_value_and_set_sign_flag(result);
                self.check_value_and_set_parity_flag(result);
                self.set_auxiliary_carry_flag((old & 0x0F) == 0);
                5
            }
            0x26 => {
                //MVI H,d8
                self.h_reg = self.read_u8_from_memory();
                7
            }
            0x27 => {
                //DAA
                let mut correction: u8 = 0;
                let mut carry = self.get_carry_flag();

                if (self.a_reg & 0x0F) > 9 || self.get_auxiliary_carry_flag()  {
                    correction |= 0x06;
                }
                if self.a_reg > 0x99 || carry {
                    correction |= 0x60;
                    carry = true;
                }

                let (result, cy) = self.a_reg.overflowing_add(correction);
                self.a_reg = result;

                self.set_carry_flag(carry || cy);
                self.check_value_and_set_zero_flag(self.a_reg);
                self.check_value_and_set_sign_flag(self.a_reg);
                self.check_value_and_set_parity_flag(self.a_reg);
                self.set_auxiliary_carry_flag((correction & 0x06) != 0);
                4

            }
            0x29 => {
                //DAD H
                self.perform_dad_operation(self.get_hl());
                10
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
            0x34 => {
                // INR M
                let addr = self.get_address_from_m_as_usize();
                let old = self.memory[addr];
                let result = old.wrapping_add(1);
                self.memory[addr] = result;

                self.check_value_and_set_zero_flag(result);
                self.check_value_and_set_sign_flag(result);
                self.check_value_and_set_parity_flag(result);
                self.set_auxiliary_carry_flag((old & 0x0F) + 1 > 0x0F);
                10
            }
            0x35 => {
                //DCR M
                let addr = self.get_address_from_m_as_usize();
                let old = self.memory[addr];
                let result = old.wrapping_sub(1);
                self.memory[addr] = result;

                self.check_value_and_set_zero_flag(result);
                self.check_value_and_set_sign_flag(result);
                self.check_value_and_set_parity_flag(result);
                self.set_auxiliary_carry_flag((old & 0x0F) == 0);
                10
            }
            0x36 => {
                //MVI M,d8
                let addr = self.get_address_from_m_as_usize();
                self.memory[addr] = self.read_u8_from_memory();
                10
            }
            0x37 => {
                //STC
                self.set_carry_flag(true);
                4
            }
            0x39 => {
                //DAD SP
                self.perform_dad_operation(self.stack_pointer);
                10
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
            0x44 => {
                //MOV B,H
                self.b_reg = self.h_reg;
                5
            }
            0x45 => {
                //MOV B,L
                self.b_reg = self.l_reg;
                5
            }
            0x46 => {
                //MOV B,M
                self.b_reg = self.memory[self.get_address_from_m_as_usize()];
                7
            }
            0x47 => {
                //MOV B,A
                self.b_reg = self.a_reg;
                5
            }
            0x48 => {
                //MOV C,B
                self.c_reg = self.b_reg;
                5
            }
            0x49 => {
                //MOV C,C
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
            0x54 => {
                //MOV D,H
                self.d_reg = self.h_reg;
                5
            }
            0x55 => {
                //MOV D,L
                self.d_reg = self.l_reg;
                5
            }
            0x56 => {
                //MOV D,M
                self.d_reg = self.memory[self.get_address_from_m_as_usize()];
                7
            }
            0x57 => {
                //MOV D,A
                self.d_reg = self.a_reg;
                5
            }
            0x58 => {
                //MOV E,B
                self.e_reg = self.b_reg;
                5
            }
            0x59 => {
                //MOV E,C
                self.e_reg = self.c_reg;
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
            0x64 => {
                //MOV H,H
                5
            }
            0x65 => {
                //MOV H,L
                self.h_reg = self.l_reg;
                5
            }
            0x66 => {
                //MOV H,M
                self.h_reg = self.memory[self.get_address_from_m_as_usize()];
                7
            }
            0x67 => {
                //MOV H,A
                self.h_reg = self.a_reg;
                5
            }
            0x68 => {
                //MOV L,B
                self.l_reg = self.b_reg;
                5
            }
            0x69 => {
                //MOV L,C
                self.l_reg = self.c_reg;
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
            0x74 => {
                //MOV M,H
                let addr = self.get_address_from_m_as_usize();
                self.memory[addr] = self.h_reg;
                7
            }
            0x75 => {
                //MOV M,L
                let addr = self.get_address_from_m_as_usize();
                self.memory[addr] = self.l_reg;
                7
            }
            0x76 => {
                // HLT
                self.halted = true;
                7
            }
            0x77 => {
                //MOV M,A
                let addr = self.get_address_from_m_as_usize();
                self.memory[addr] = self.a_reg;
                7
            }
            0x78 => {
                //MOV A,B
                self.a_reg = self.b_reg;
                5
            }
            0x79 => {
                //MOV A,C
                self.a_reg = self.c_reg;
                5
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
            0x84 => {
                //ADD H
                self.perform_u8_addition(self.h_reg);
                4
            }
            0x85 => {
                //ADD L
                self.perform_u8_addition(self.l_reg);
                4
            }
            0x86 => {
                //ADD M
                let addr = self.get_address_from_m_as_usize();
                self.perform_u8_addition(self.memory[addr]);
                7
            }
            0x87 => {
                //ADD A
                self.perform_u8_addition(self.a_reg);
                4
            }
            0x88 => {
                //ADC B
                self.perform_u8_addition_with_carry(self.b_reg);
                4
            }
            0x89 => {
                //ADC C
                self.perform_u8_addition_with_carry(self.c_reg);
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
            0x94 => {
                //SUB H
                self.perform_u8_subtraction(self.h_reg);
                4
            }
            0x95 => {
                //SUB L
                self.perform_u8_subtraction(self.l_reg);
                4
            }
            0x96 => {
                //SUB M
                let addr = self.get_address_from_m_as_usize();
                self.perform_u8_subtraction(self.memory[addr]);
                7
            }
            0x97 => {
                self.perform_u8_subtraction(self.a_reg);
                4
            }
            0x98 => {
                //SBB B
                self.perform_u8_subtraction_with_borrow(self.b_reg);
                4
            }
            0x99 => {
                //SBB C
                self.perform_u8_subtraction_with_borrow(self.c_reg);
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
            0xA4 => {
                //ANA H
                self.perform_and_operation(self.h_reg);
                4
            }
            0xA5 => {
                //ANA L
                self.perform_and_operation(self.l_reg);
                4
            }
            0xA6 => {
                //ANA M
                let addr = self.get_address_from_m_as_usize();
                self.perform_and_operation(self.memory[addr]);
                7
            }
            0xA7 => {
                //ANA A
                self.perform_and_operation(self.a_reg);
                4
            }
            0xA8 => {
                //XRA B
                self.perform_xra_operation(self.b_reg);
                4
            }
            0xA9 => {
                //XRA C
                self.perform_xra_operation(self.c_reg);
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
            0xB4 => {
                //ORA H
                self.perform_or_operation(self.h_reg);
                4
            }
            0xB5 => {
                //ORA L
                self.perform_or_operation(self.l_reg);
                4
            }
            0xB6 => {
                //ORA M
                let addr = self.get_address_from_m_as_usize();
                self.perform_or_operation(self.memory[addr]);
                7
            }
            0xB7 => {
                //ORA A
                self.perform_or_operation(self.a_reg);
                4
            }
            0xB8 => {
                //CMP B
                self.perform_compare_operation(self.b_reg);
                4
            }
            0xB9 => {
                //CMP C
                self.perform_compare_operation(self.c_reg);
                4
            }
            0xC0 => {
                //RNZ
                if !self.get_zero_flag(){
                    self.program_counter = self.pop_stack_u16();
                    11
                } else {
                    5
                }
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
                }
                11
            }
            0xC3 => {
                //JMP a16
                self.program_counter = self.read_u16_from_memory();
                10
            }
            0xC4 => {
                //CNZ a16
                let address = self.read_u16_from_memory();

                if !self.get_zero_flag() {
                    self.push_stack_u16(self.program_counter);
                    self.program_counter = address;
                    17
                } else {
                    11
                }

            }
            0xC5 => {
                //PUSH B
                self.push_stack_u16(self.get_bc());
                11

            }
            0xC6 => {
                //ADI d8
                let value = self.read_u8_from_memory();
                self.perform_u8_addition(value);
                7
            }
            0xC7 => {
                //RST 0
                self.push_stack_u16(self.program_counter);
                self.program_counter = 0x0000;
                11
            }
            0xC8 => {
                //RZ
                if self.get_zero_flag(){
                    self.program_counter = self.pop_stack_u16();
                    11
                } else {
                    5
                }
            }
            0xD0 => {
                //RNC
                if !self.get_carry_flag(){
                    self.program_counter = self.pop_stack_u16();
                    11
                } else {
                    5
                }
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
                }
                11
            }
            0xD3 => {
                //OUT d8
                let device = self.read_u8_from_memory();
                io_handler::handle_output(device, self.a_reg);
                10

            }
            0xD4 => {
                //CNC a16
                let address = self.read_u16_from_memory();

                if !self.get_carry_flag() {
                    self.push_stack_u16(self.program_counter);
                    self.program_counter = address;
                    17
                } else {
                    11
                }
            }
            0xD5 => {
                //PUSH D
                self.push_stack_u16(self.get_de());
                11
            }
            0xD6 => {
                //SUI d8
                let value = self.read_u8_from_memory();
                self.perform_u8_subtraction(value);
                7
            }
            0xD7 => {
                //RST 2
                self.push_stack_u16(self.program_counter);
                self.program_counter = 0x0010;
                11
            }
            0xD8 => {
                //RC
                if self.get_carry_flag(){
                    self.program_counter = self.pop_stack_u16();
                    11
                } else {
                    5
                }
            }
            0xE0 => {
                //RPO
                if !self.get_parity_flag() {
                    self.program_counter = self.pop_stack_u16();
                    11
                } else {
                    5
                }
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
            0xE4 => {
                //CPO a16
                let address = self.read_u16_from_memory();

                if !self.get_parity_flag() {
                    self.push_stack_u16(self.program_counter);
                    self.program_counter = address;
                    17
                } else {
                    11
                }
            }
            0xE5 => {
                //PUSH H
                self.push_stack_u16(self.get_hl());
                11
            }
            0xE6 => {
                //ANI d8
                let value = self.read_u8_from_memory();
                self.perform_and_operation(value);
                7
            }
            0xE7 => {
                //RST 4
                self.push_stack_u16(self.program_counter);
                self.program_counter = 0x0020;
                11
            }
            0xE8 => {
                //RPE
                if self.get_parity_flag() {
                    self.program_counter = self.pop_stack_u16();
                    11
                } else {
                    5
                }
            }
            0xF0 => {
                //RP
                if !self.get_sign_flag() {
                    self.program_counter = self.pop_stack_u16();
                    11
                } else {
                    5
                }
            }
            0xF1 => {
                //POP PSW
                let value = self.pop_stack_u16();
                let flags = (value & 0x00FF) as u8;
                let a = (value >> 8) as u8;

                self.flags = flags & 0b1101_0111;
                self.flags |= 0b0000_0010;
                self.a_reg = a;
                10
            }
            0xF2 => {
                //JP a16
                let address = self.read_u16_from_memory();
                if !self.get_sign_flag() {
                    self.program_counter = address;
                }
                11
            }
            0xF3 => {
                //DI
                self.interrupts_enabled = false;
                4
            }
            0xF4 => {
                //CP a16
                let address = self.read_u16_from_memory();

                if !self.get_sign_flag() {
                    self.push_stack_u16(self.program_counter);
                    self.program_counter = address;
                    17
                } else {
                    11
                }
            }
            0xF5 => {
                //PUSH PSW
                let flags = (self.flags & 0b1101_0111) | 0b0000_0010;
                let psw = ((self.a_reg as u16) << 8) | flags as u16;
                self.push_stack_u16(psw);
                11
            }
            0xF6 => {
                //ORI d8
                let value = self.read_u8_from_memory();
                self.perform_or_operation(value);
                7
            }
            0xF7 => {
                //RST 6
                self.push_stack_u16(self.program_counter);
                self.program_counter = 0x0030;
                11
            }
            0xF8 => {
                //RM
                if self.get_sign_flag() {
                    self.program_counter = self.pop_stack_u16();
                    11
                } else {
                    5
                }
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

    fn get_sign_flag(&self) -> bool{
        self.flags & 0b1000_0000 != 0
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
    fn get_auxiliary_carry_flag(&self) -> bool{
        self.flags & 0b0001_0000 != 0
    }

    fn check_value_and_set_sign_flag(&mut self, value: u8){
        self.set_sign_flag(value & 0b1000_0000 == 0b1000_0000)
    }

    fn check_value_and_set_zero_flag(&mut self, value: u8){
        self.set_zero_flag(value == 0)
    }

    fn check_value_and_set_parity_flag(&mut self, value: u8){
        self.set_parity_flag(value.count_ones() % 2 == 0)
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

    fn push_stack_u16(&mut self, value: u16){
        let hi = (value >> 8) as u8;
        let lo = value as u8;
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.memory[self.stack_pointer as usize] = hi;
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.memory[self.stack_pointer as usize] = lo;
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
        self.check_value_and_set_sign_flag(self.a_reg);
        self.check_value_and_set_zero_flag(self.a_reg);
        self.check_value_and_set_parity_flag(self.a_reg);
    }
    fn perform_dad_operation(&mut self, value: u16){
        let hl = self.get_hl();
        let (result, carry) = hl.overflowing_add(value);

        self.set_hl(result);
        self.set_carry_flag(carry);
    }

    fn perform_u8_subtraction(&mut self, value: u8) {
        let (result, borrow) = self.a_reg.overflowing_sub(value);
        self.set_carry_flag(borrow);

        let aux_carry = (self.a_reg & 0x0F) < (value & 0x0F);
        self.set_auxiliary_carry_flag(aux_carry);

        self.a_reg = result;
        self.check_value_and_set_sign_flag(self.a_reg);
        self.check_value_and_set_zero_flag(self.a_reg);
        self.check_value_and_set_parity_flag(self.a_reg);
    }

     fn perform_and_operation(&mut self, value: u8){
        self.a_reg &= value;
        self.set_carry_flag(false);
        self.check_value_and_set_zero_flag(self.a_reg);
        self.check_value_and_set_sign_flag(self.a_reg);
        self.check_value_and_set_parity_flag(self.a_reg);
        self.set_auxiliary_carry_flag(true); //TODO: sprawdzi czy na pewno
    }

     fn perform_or_operation(&mut self, value: u8){
        self.a_reg |= value;
        self.set_carry_flag(false);
        self.check_value_and_set_zero_flag(self.a_reg);
        self.check_value_and_set_sign_flag(self.a_reg);
        self.check_value_and_set_parity_flag(self.a_reg);
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

    fn perform_u8_addition_with_carry(&mut self, value: u8) {
        let carry = self.get_carry_flag() as u8;

        let (tmp, carry1) = self.a_reg.overflowing_add(value);
        let (result, carry2) = tmp.overflowing_add(carry);

        self.set_carry_flag(carry1 || carry2);
        let ac = ((self.a_reg & 0x0F) + (value & 0x0F) + carry) > 0x0F;
        self.set_auxiliary_carry_flag(ac);
        self.a_reg = result;
        self.check_value_and_set_zero_flag(result);
        self.check_value_and_set_sign_flag(result);
        self.check_value_and_set_parity_flag(result);
    }

    fn perform_u8_subtraction_with_borrow(&mut self, value: u8) {
        let carry = self.get_carry_flag() as u8;

        let (tmp, borrow1) = self.a_reg.overflowing_sub(value);
        let (result, borrow2) = tmp.overflowing_sub(carry);

        self.set_carry_flag(borrow1 || borrow2);
        let ac = (self.a_reg & 0x0F) < ((value & 0x0F) + carry);
        self.set_auxiliary_carry_flag(ac);
        self.a_reg = result;
        self.check_value_and_set_zero_flag(result);
        self.check_value_and_set_sign_flag(result);
        self.check_value_and_set_parity_flag(result);
    }

    fn perform_xra_operation(&mut self, value: u8) {
        self.a_reg ^= value;
        self.set_carry_flag(false);
        self.set_auxiliary_carry_flag(false);
        self.check_value_and_set_zero_flag(self.a_reg);
        self.check_value_and_set_sign_flag(self.a_reg);
        self.check_value_and_set_parity_flag(self.a_reg);
    }

    fn perform_compare_operation(&mut self, value: u8) {
        let (result, borrow) = self.a_reg.overflowing_sub(value);

        self.set_carry_flag(borrow);
        let ac = (self.a_reg & 0x0F) < (value & 0x0F);
        self.set_auxiliary_carry_flag(ac);
        self.check_value_and_set_zero_flag(result);
        self.check_value_and_set_sign_flag(result);
        self.check_value_and_set_parity_flag(result);
    }
}