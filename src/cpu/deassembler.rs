pub(crate) fn deassemble(opcode: u8, lo: u8, hi: u8) -> String {
    let imm16 = || format!("{:#06X}", u16::from_le_bytes([lo, hi]));
    let imm8 = || format!("{:#04X}", lo);

    match opcode {
        0x00 => "NOP".to_string(),
        0x01 => format!("LXI B,{}", imm16()),
        0x02 => "STAX B".to_string(),
        0x03 => "INX B".to_string(),
        0x04 => "INR B".to_string(),
        0x05 => "DCR B".to_string(),
        0x06 => format!("MVI B,{}", imm8()),
        0x07 => "RLC".to_string(),
        0x08 => "NOP***".to_string(),
        0x09 => "DAD B".to_string(),
        0x0A => "LDAX B".to_string(),
        0x0B => "DCX B".to_string(),
        0x0C => "INR C".to_string(),
        0x0D => "DCR C".to_string(),
        0x0E => format!("MVI C,{}", imm8()),
        0x0F => "RRC".to_string(),

        0x10 => "NOP***".to_string(),
        0x11 => format!("LXI D,{}", imm16()),
        0x12 => "STAX D".to_string(),
        0x13 => "INX D".to_string(),
        0x14 => "INR D".to_string(),
        0x15 => "DCR D".to_string(),
        0x16 => format!("MVI D,{}", imm8()),
        0x17 => "RAL".to_string(),
        0x18 => "NOP***".to_string(),
        0x19 => "DAD D".to_string(),
        0x1A => "LDAX D".to_string(),
        0x1B => "DCX D".to_string(),
        0x1C => "INR E".to_string(),
        0x1D => "DCR E".to_string(),
        0x1E => format!("MVI E,{}", imm8()),
        0x1F => "RAR".to_string(),

        0x20 => "NOP***".to_string(),
        0x21 => format!("LXI H,{}", imm16()),
        0x22 => format!("SHLD {}", imm16()),
        0x23 => "INX H".to_string(),
        0x24 => "INR H".to_string(),
        0x25 => "DCR H".to_string(),
        0x26 => format!("MVI H,{}", imm8()),
        0x27 => "DAA".to_string(),
        0x28 => "NOP***".to_string(),
        0x29 => "DAD H".to_string(),
        0x2A => format!("LHLD {}", imm16()),
        0x2B => "DCX H".to_string(),
        0x2C => "INR L".to_string(),
        0x2D => "DCR L".to_string(),
        0x2E => format!("MVI L,{}", imm8()),
        0x2F => "CMA".to_string(),

        0x30 => "NOP***".to_string(),
        0x31 => format!("LXI SP,{}", imm16()),
        0x32 => format!("STA {}", imm16()),
        0x33 => "INX SP".to_string(),
        0x34 => "INR M".to_string(),
        0x35 => "DCR M".to_string(),
        0x36 => format!("MVI M,{}", imm8()),
        0x37 => "STC".to_string(),
        0x38 => "NOP***".to_string(),
        0x39 => "DAD SP".to_string(),
        0x3A => format!("LDA {}", imm16()),
        0x3B => "DCX SP".to_string(),
        0x3C => "INR A".to_string(),
        0x3D => "DCR A".to_string(),
        0x3E => format!("MVI A,{}", imm8()),
        0x3F => "CMC".to_string(),

        0x76 => "HLT".to_string(),

        0x40..=0x7F => {
            let d = (opcode >> 3) & 0x07;
            let s = opcode & 0x07;
            let reg = ["B","C","D","E","H","L","M","A"];
            (format!("MOV {},{}", reg[d as usize], reg[s as usize]))
        },

        0x80..=0x87 => (format!("ADD {}", match opcode & 0x07 {
            0 => "B", 1 => "C", 2 => "D", 3 => "E", 4 => "H", 5 => "L", 6 => "M", _ => "A"
        })),
        0x88..=0x8F => (format!("ADC {}", match opcode & 0x07 {
            0 => "B",1=>"C",2=>"D",3=>"E",4=>"H",5=>"L",6=>"M",_=>"A"
        })),
        0x90..=0x97 => (format!("SUB {}", match opcode & 0x07 {
            0 => "B",1=>"C",2=>"D",3=>"E",4=>"H",5=>"L",6=>"M",_=>"A"
        })),
        0x98..=0x9F => (format!("SBB {}", match opcode & 0x07 {
            0 => "B",1=>"C",2=>"D",3=>"E",4=>"H",5=>"L",6=>"M",_=>"A"
        })),

        0xA0..=0xA7 => (format!("ANA {}", match opcode & 0x07 {
            0 => "B",1=>"C",2=>"D",3=>"E",4=>"H",5=>"L",6=>"M",_=>"A"
        })),
        0xA8..=0xAF => (format!("XRA {}", match opcode & 0x07 {
            0 => "B",1=>"C",2=>"D",3=>"E",4=>"H",5=>"L",6=>"M",_=>"A"
        })),
        0xB0..=0xB7 => (format!("ORA {}", match opcode & 0x07 {
            0 => "B",1=>"C",2=>"D",3=>"E",4=>"H",5=>"L",6=>"M",_=>"A"
        })),
        0xB8..=0xBF => (format!("CMP {}", match opcode & 0x07 {
            0 => "B",1=>"C",2=>"D",3=>"E",4=>"H",5=>"L",6=>"M",_=>"A"
        })),

        0xC0 => "RNZ".to_string(),
        0xC1 => "POP B".to_string(),
        0xC2 => format!("JNZ {}", imm16()),
        0xC3 => format!("JMP {}", imm16()),
        0xC4 => format!("CNZ {}", imm16()),
        0xC5 => "PUSH B".to_string(),
        0xC6 => format!("ADI {}", imm8()),
        0xC7 => "RST 0".to_string(),
        0xC8 => "RZ".to_string(),
        0xC9 => "RET".to_string(),
        0xCA => format!("JZ {}", imm16()),
        0xCB => "JMP a16*** -> NOP".to_string(),
        0xCC => format!("CZ {}", imm16()),
        0xCD => format!("CALL {}", imm16()),
        0xCE => format!("ACI {}", imm8()),
        0xCF => "RST 1".to_string(),

        0xD0 => "RNC".to_string(),
        0xD1 => "POP D".to_string(),
        0xD2 => format!("JNC {}", imm16()),
        0xD3 => format!("OUT {}", imm8()),
        0xD4 => format!("CNC {}", imm16()),
        0xD5 => "PUSH D".to_string(),
        0xD6 => format!("SUI {}", imm8()),
        0xD7 => "RST 2".to_string(),
        0xD8 => "RC".to_string(),
        0xD9 => "RET*** -> NOP".to_string(),
        0xDA => format!("JC {}",imm16()),
        0xDB => format!("IN {}", imm8()),
        0xDC => format!("CC {}", imm16()),
        0xDD => "CALL*** -> NOP".to_string(),
        0xDE => format!("SBI {}", imm8()),
        0xDF => "RST 3".to_string(),

        0xE0 => "RPO".to_string(),
        0xE1 => "POP H".to_string(),
        0xE2 => format!("JPO {}", imm16()),
        0xE3 => "XTHL".to_string(),
        0xE4 => format!("CPO {}", imm16()),
        0xE5 => "PUSH H".to_string(),
        0xE6 => format!("ANI {}", imm8()),
        0xE7 => "RST 4".to_string(),
        0xE8 => "RPE".to_string(),
        0xE9 => "PCHL".to_string(),
        0xEA => format!("JPE {}", imm16()),
        0xEB => "XCHG".to_string(),
        0xEC => format!("CPE {}", imm16()),
        0xED => "CALL*** -> NOP".to_string(),
        0xEE => format!("XRI {}", imm8()),
        0xEF => "RST 5".to_string(),

        0xF0 => "RP".to_string(),
        0xF1 => "POP PSW".to_string(),
        0xF2 => format!("JP {}", imm16()),
        0xF3 => "DI".to_string(),
        0xF4 => format!("CP {}", imm16()),
        0xF5 => "PUSH PSW".to_string(),
        0xF6 => format!("ORI {}", imm8()),
        0xF7 => "RST 6".to_string(),
        0xF8 => "RM".to_string(),
        0xF9 => "SPHL".to_string(),
        0xFA => format!("JM {}", imm16()),
        0xFB => "EI".to_string(),
        0xFC => format!("CM {}", imm16()),
        0xFD => "CALL*** -> NOP".to_string(),
        0xFE => format!("CPI {}", imm8()),
        0xFF => "RST 7".to_string(),
    }
}
