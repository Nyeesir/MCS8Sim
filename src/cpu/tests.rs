use crate::cpu;

use super::*;

fn get_reg(cpu: &Cpu, reg: u8) -> u8 {
    match reg {
        0 => cpu.b_reg,
        1 => cpu.c_reg,
        2 => cpu.d_reg,
        3 => cpu.e_reg,
        4 => cpu.h_reg,
        5 => cpu.l_reg,
        7 => cpu.a_reg,
        _ => unreachable!(),
    }
}

fn set_reg(cpu: &mut Cpu, reg: u8, val: u8) {
    match reg {
        0 => cpu.b_reg = val,
        1 => cpu.c_reg = val,
        2 => cpu.d_reg = val,
        3 => cpu.e_reg = val,
        4 => cpu.h_reg = val,
        5 => cpu.l_reg = val,
        7 => cpu.a_reg = val,
        _ => unreachable!(),
    }
}

#[test]
fn mov_all_register_combinations() {
    for dst in 0u8..=7 {
        for src in 0u8..=7 {
            if dst == 6 || src == 6 {
                continue;
            }

            let opcode = 0x40 | (dst << 3) | src;

            let mut cpu = Cpu::new();
            cpu.memory[0] = opcode;

            set_reg(&mut cpu, src, 0xAB);

            cpu.step();

            let result = get_reg(&cpu, dst);
            assert_eq!(
                result,
                0xAB,
                "MOV failed: dst={}, src={}, opcode=0x{:02X}",
                dst,
                src,
                opcode
            );
        }
    }
}

#[test]
fn mov_r_from_m() {
    for dst in 0u8..=7 {
        if dst == 6 {
            continue;
        }

        let opcode = 0x46 | (dst << 3);

        let mut cpu = Cpu::new();
        cpu.memory[0] = opcode;

        cpu.h_reg = 0x12;
        cpu.l_reg = 0x34;
        let addr = 0x1234;

        cpu.memory[addr] = 0xCD;

        cpu.step();

        assert_eq!(
            get_reg(&cpu, dst),
            0xCD,
            "MOV r,M failed: dst={}, opcode=0x{:02X}",
            dst,
            opcode
        );
    }
}

#[test]
fn mov_m_from_r() {
    for src in 0u8..=7 {
        if src == 6 || src == 4 || src == 5 {
            continue;
        }

        let opcode = 0x70 | src;

        let mut cpu = Cpu::new();
        cpu.memory[0] = opcode;

        cpu.h_reg = 0x20;
        cpu.l_reg = 0x10;
        let addr = 0x2010;

        set_reg(&mut cpu, src, 0xEF);

        cpu.step();

        assert_eq!(
            cpu.memory[addr],
            0xEF,
            "MOV M,r failed: src={}, opcode=0x{:02X}",
            src,
            opcode
        );
    }
}

#[test]
fn mov_m_h(){
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x74;

    cpu.h_reg = 0x20;
    cpu.l_reg = 0x10;
    let addr = 0x2010;

    cpu.step();

    assert_eq!(
        cpu.memory[addr],
        cpu.h_reg,
        "MOV M,H failed, opcode=0x74",
    );
}

#[test]
fn mov_m_l(){
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x75;

    cpu.h_reg = 0x20;
    cpu.l_reg = 0x10;
    let addr = 0x2010;

    cpu.step();

    assert_eq!(
        cpu.memory[addr],
        cpu.l_reg,
        "MOV M,L failed, opcode=0x74",
    );
}

#[test]
fn add_all_registers() {
    for src in 0u8..=7 {
        if src == 6 || src == 7{
            continue; // M
        }

        let opcode = 0x80 | src; // ADD r

        let mut cpu = Cpu::new();
        cpu.memory[0] = opcode;

        cpu.a_reg = 0x10;
        set_reg(&mut cpu, src, 0x22);

        cpu.step();

        assert_eq!(
            cpu.a_reg,
            0x32,
            "ADD r failed: src={}, opcode=0x{:02X}",
            src,
            opcode
        );
    }
}

fn add_a() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x87;

    cpu.a_reg = 0x10;

    cpu.step();

    assert_eq!(
        cpu.a_reg,
        0x20,
        "ADD A failed, opcode=087",
    );
}

#[test]
fn add_m() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x86; // ADD M

    cpu.a_reg = 0x40;
    cpu.h_reg = 0x12;
    cpu.l_reg = 0x34;

    cpu.memory[0x1234] = 0x20;

    cpu.step();

    assert_eq!(cpu.a_reg, 0x60, "ADD M failed");
}

#[test]
fn add_sets_zero_flag() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x80; // ADD B

    cpu.a_reg = 0x00;
    cpu.b_reg = 0x00;

    cpu.step();

    assert!(cpu.get_zero_flag(), "Zero flag not set");
}

#[test]
fn add_sets_sign_flag() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x80; // ADD B

    cpu.a_reg = 0x40;
    cpu.b_reg = 0x40; // 0x80

    cpu.step();

    assert!(cpu.get_sign_flag(), "Sign flag not set");
}

#[test]
fn add_sets_carry_flag() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x80; // ADD B

    cpu.a_reg = 0xF0;
    cpu.b_reg = 0x30; // overflow

    cpu.step();

    assert!(cpu.get_carry_flag(), "Carry flag not set");
}

#[test]
fn add_sets_auxiliary_carry_flag() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x80; // ADD B

    cpu.a_reg = 0x0F;
    cpu.b_reg = 0x01;

    cpu.step();

    assert!(
        cpu.get_auxiliary_carry_flag(),
        "Auxiliary Carry flag not set"
    );
}

#[test]
fn add_sets_parity_flag() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x80; // ADD B

    cpu.a_reg = 0x01;
    cpu.b_reg = 0x01; // result = 0x02 (1 bit set -> odd)

    cpu.step();

    assert!(
        !cpu.get_parity_flag(),
        "Parity flag incorrect"
    );
}

#[test]
fn sub_all_registers() {
    for src in 0u8..=7 {
        if src == 6 || src == 7 {
            continue; // M i A osobno
        }

        let opcode = 0x90 | src; // SUB r

        let mut cpu = Cpu::new();
        cpu.memory[0] = opcode;

        cpu.a_reg = 0x50;
        set_reg(&mut cpu, src, 0x10);

        cpu.step();

        assert_eq!(
            cpu.a_reg,
            0x40,
            "SUB r failed: src={}, opcode=0x{:02X}",
            src,
            opcode
        );
    }
}

#[test]
fn sub_m() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x96; // SUB M

    cpu.a_reg = 0x30;
    cpu.h_reg = 0x12;
    cpu.l_reg = 0x34;

    cpu.memory[0x1234] = 0x20;

    cpu.step();

    assert_eq!(cpu.a_reg, 0x10, "SUB M failed");
}

#[test]
fn sub_a_8080() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x97; // SUB A

    cpu.a_reg = 0x3C;

    cpu.step();

    assert_eq!(
        cpu.a_reg,
        0x00,
        "SUB A failed, opcode=0x97"
    );
}

#[test]
fn sub_sets_zero_flag() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x90; // SUB B

    cpu.a_reg = 0x20;
    cpu.b_reg = 0x20;

    cpu.step();

    assert!(cpu.get_zero_flag(), "Zero flag not set");
}

#[test]
fn sub_sets_sign_flag() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x90; // SUB B

    cpu.a_reg = 0x10;
    cpu.b_reg = 0x20; // wynik ujemny

    cpu.step();

    assert!(cpu.get_sign_flag(), "Sign flag not set");
}

#[test]
fn sub_sets_carry_flag_on_borrow() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x90; // SUB B

    cpu.a_reg = 0x10;
    cpu.b_reg = 0x20;

    cpu.step();

    assert!(cpu.get_carry_flag(), "Carry flag not set on borrow");
}

#[test]
fn sub_sets_auxiliary_carry_flag() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x90; // SUB B

    cpu.a_reg = 0x10;
    cpu.b_reg = 0x01; // borrow z bitu 4

    cpu.step();

    assert!(
        cpu.get_auxiliary_carry_flag(),
        "Auxiliary Carry flag not set"
    );
}

#[test]
fn sub_sets_parity_flag() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x90; // SUB B

    cpu.a_reg = 0x05;
    cpu.b_reg = 0x01; // 0x04 → 1 bit → odd

    cpu.step();

    assert!(
        !cpu.get_parity_flag(),
        "Parity flag incorrect"
    );
}

#[test]
fn ana_all_registers() {
    for src in 0u8..=7 {
        if src == 6 || src == 7 {
            continue; // M i A osobno
        }

        let opcode = 0xA0 | src; // ANA r

        let mut cpu = Cpu::new();
        cpu.memory[0] = opcode;

        cpu.a_reg = 0b1100_1100;
        set_reg(&mut cpu, src, 0b1010_1010);

        cpu.step();

        assert_eq!(
            cpu.a_reg,
            0b1000_1000,
            "ANA r failed: src={}, opcode=0x{:02X}",
            src,
            opcode
        );
    }
}

#[test]
fn ana_m() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0xA6; // ANA M

    cpu.a_reg = 0b1111_0000;
    cpu.h_reg = 0x12;
    cpu.l_reg = 0x34;
    cpu.memory[0x1234] = 0b0011_0011;

    cpu.step();

    assert_eq!(cpu.a_reg, 0b0011_0000, "ANA M failed");
}

#[test]
fn ana_a() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0xA7; // ANA A

    cpu.a_reg = 0x5A;

    cpu.step();

    assert_eq!(cpu.a_reg, 0x5A, "ANA A failed");
}

#[test]
fn ana_flags() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0xA0; // ANA B

    cpu.a_reg = 0xF0;
    cpu.b_reg = 0x0F;

    cpu.step();

    assert!(cpu.get_zero_flag(), "ANA: Zero flag not set");
    assert!(!cpu.get_sign_flag(), "ANA: Sign flag incorrect");
    assert!(cpu.get_parity_flag(), "ANA: Parity flag incorrect");
    assert!(!cpu.get_carry_flag(), "ANA: Carry flag should be reset");
    assert!(cpu.get_auxiliary_carry_flag(), "ANA: AC should be set");
}

#[test]
fn ora_all_registers() {
    for src in 0u8..=7 {
        if src == 6 || src == 7 {
            continue; // M i A osobno
        }

        let opcode = 0xB0 | src; // ORA r

        let mut cpu = Cpu::new();
        cpu.memory[0] = opcode;

        cpu.a_reg = 0b1100_0000;
        set_reg(&mut cpu, src, 0b0011_0011);

        cpu.step();

        assert_eq!(
            cpu.a_reg,
            0b1111_0011,
            "ORA r failed: src={}, opcode=0x{:02X}",
            src,
            opcode
        );
    }
}

#[test]
fn ora_m() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0xB6; // ORA M

    cpu.a_reg = 0b1000_0000;
    cpu.h_reg = 0x12;
    cpu.l_reg = 0x34;
    cpu.memory[0x1234] = 0b0000_1111;

    cpu.step();

    assert_eq!(cpu.a_reg, 0b1000_1111, "ORA M failed");
}

#[test]
fn ora_a() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0xB7; // ORA A

    cpu.a_reg = 0x3C;

    cpu.step();

    assert_eq!(cpu.a_reg, 0x3C, "ORA A failed");
}

#[test]
fn ora_flags() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0xB0; // ORA B

    cpu.a_reg = 0x00;
    cpu.b_reg = 0x00;

    cpu.step();

    assert!(cpu.get_zero_flag(), "ORA: Zero flag not set");
    assert!(!cpu.get_sign_flag(), "ORA: Sign flag incorrect");
    assert!(cpu.get_parity_flag(), "ORA: Parity flag incorrect");
    assert!(!cpu.get_carry_flag(), "ORA: Carry flag should be reset");
    assert!(!cpu.get_auxiliary_carry_flag(), "ORA: AC should be reset");
}

#[test]
fn sbb() {
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x9D;
    cpu.set_carry_flag(true);
    cpu.l_reg = 0x2;
    cpu.a_reg = 0x4;
    cpu.step();
    assert_eq!(cpu.a_reg, 0x1);
    assert!(!cpu.get_carry_flag());
}

#[test]
fn daa(){
    let mut cpu = Cpu::new();
    cpu.memory[0] = 0x27;
    cpu.a_reg = 0x9B;
    cpu.step();
    assert_eq!(cpu.a_reg, 0x1);
    assert!(cpu.get_carry_flag());
    assert!(cpu.get_auxiliary_carry_flag());
}

