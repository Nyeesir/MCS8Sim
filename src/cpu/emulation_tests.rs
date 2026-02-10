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

#[test]
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

#[test]
fn stack_push_pop_u16() {
    let mut cpu = Cpu::new();

    cpu.stack_pointer = 0x1000;

    cpu.push_stack_u16(0x1234);
    assert_eq!(cpu.stack_pointer, 0x0FFE);

    let value = cpu.pop_stack_u16();
    assert_eq!(value, 0x1234);
    assert_eq!(cpu.stack_pointer, 0x1000);
}

#[test]
fn stack_lifo_order() {
    let mut cpu = Cpu::new();
    cpu.stack_pointer = 0x2000;

    cpu.push_stack_u16(0xAAAA);
    cpu.push_stack_u16(0xBBBB);

    let v1 = cpu.pop_stack_u16();
    let v2 = cpu.pop_stack_u16();

    assert_eq!(v1, 0xBBBB);
    assert_eq!(v2, 0xAAAA);
}

#[test]
fn call_and_ret_restore_pc() {
    let mut cpu = Cpu::new();

    // program:
    // 0000: CD 05 00   CALL 0005
    // 0003: 76         HLT
    // 0005: C9         RET

    cpu.memory[0x0000] = 0xCD;
    cpu.memory[0x0001] = 0x05;
    cpu.memory[0x0002] = 0x00;

    cpu.memory[0x0003] = 0x76; // HLT

    cpu.memory[0x0005] = 0xC9; // RET

    cpu.program_counter = 0x0000;
    cpu.stack_pointer = 0x2000;

    cpu.step(); // CALL
    assert_eq!(cpu.program_counter, 0x0005);

    cpu.step(); // RET
    assert_eq!(cpu.program_counter, 0x0003);
}

#[test]
fn rst_acts_like_call() {
    let mut cpu = Cpu::new();

    cpu.program_counter = 0x0100;
    cpu.stack_pointer = 0x3000;

    cpu.memory[0x0100] = 0xC7; // RST 0

    cpu.step();

    assert_eq!(cpu.program_counter, 0x0000);

    let ret = cpu.pop_stack_u16();
    assert_eq!(ret, 0x0101);
}

#[test]
fn cmp_sets_zero_flag_correctly() {
    let mut cpu = Cpu::new();

    cpu.a_reg = 0x42;
    cpu.perform_compare_operation(0x42);

    assert!(cpu.get_zero_flag());
    assert!(!cpu.get_carry_flag());
}

#[test]
fn cmp_sets_carry_when_a_less_than_value() {
    let mut cpu = Cpu::new();

    cpu.a_reg = 0x10;
    cpu.perform_compare_operation(0x20);

    assert!(cpu.get_carry_flag()); // borrow
    assert!(!cpu.get_zero_flag());
}

#[test]
fn cmp_no_carry_when_a_greater() {
    let mut cpu = Cpu::new();

    cpu.a_reg = 0x30;
    cpu.perform_compare_operation(0x20);

    assert!(!cpu.get_carry_flag());
    assert!(!cpu.get_zero_flag());
}

#[test]
fn sub_sets_zero_and_carry() {
    let mut cpu = Cpu::new();

    cpu.a_reg = 0x10;
    cpu.perform_u8_subtraction(0x10);

    assert_eq!(cpu.a_reg, 0x00);
    assert!(cpu.get_zero_flag());
    assert!(!cpu.get_carry_flag());
}

#[test]
fn sbb_with_borrow() {
    let mut cpu = Cpu::new();

    cpu.a_reg = 0x04;
    cpu.l_reg = 0x02;
    cpu.set_carry_flag(true);

    cpu.perform_u8_subtraction_with_borrow(cpu.l_reg);

    assert_eq!(cpu.a_reg, 0x01);
    assert!(!cpu.get_carry_flag());
}

#[test]
fn jz_taken_when_zero() {
    let mut cpu = Cpu::new();

    cpu.set_zero_flag(true);

    cpu.memory[0] = 0xCA; // JZ
    cpu.memory[1] = 0x34;
    cpu.memory[2] = 0x12;

    cpu.step();

    assert_eq!(cpu.program_counter, 0x1234);
}

#[test]
fn jz_not_taken_when_not_zero() {
    let mut cpu = Cpu::new();

    cpu.set_zero_flag(false);

    cpu.memory[0] = 0xCA;
    cpu.memory[1] = 0x34;
    cpu.memory[2] = 0x12;

    cpu.step();

    assert_eq!(cpu.program_counter, 0x0003);
}

#[test]
fn jnz_taken_when_not_zero() {
    let mut cpu = Cpu::new();

    cpu.set_zero_flag(false);

    cpu.memory[0] = 0xC2; // JNZ
    cpu.memory[1] = 0x78;
    cpu.memory[2] = 0x56;

    cpu.step();

    assert_eq!(cpu.program_counter, 0x5678);
}

#[test]
fn jc_taken_when_carry() {
    let mut cpu = Cpu::new();

    cpu.set_carry_flag(true);

    cpu.memory[0] = 0xDA; // JC
    cpu.memory[1] = 0x00;
    cpu.memory[2] = 0x20;

    cpu.step();

    assert_eq!(cpu.program_counter, 0x2000);
}

#[test]
fn loop_breaks_when_zero_set() {
    let mut cpu = Cpu::new();

    // 0000: CMP B
    // 0001: JNZ 0000
    cpu.memory[0] = 0xB8; // CMP B
    cpu.memory[1] = 0xC2; // JNZ
    cpu.memory[2] = 0x00;
    cpu.memory[3] = 0x00;

    cpu.a_reg = 0x10;
    cpu.b_reg = 0x10; // CMP => Z=1

    cpu.step(); // CMP
    cpu.step(); // JNZ

    assert_eq!(cpu.program_counter, 0x0004); // loop broken
}

// #[test]
// fn cpu_runs_mcs8_bios() {
//     use std::fs;
//
//     let mut cpu = Cpu::new();
//
//     let bios = fs::read("src/bios.bin")
//         .expect("cannot read bios.bin");
//
//     cpu.memory[0x0000..bios.len()].copy_from_slice(&bios);
//     cpu.program_counter = 0x0000;
//
//     let mut steps: u64 = 0;
//     const MAX_STEPS: u64 = 10_000;
//
//     println!("\n--- MCS-8 BIOS OUTPUT START ---\n");
//
//     let mut operations: std::vec::Vec<String> = Vec::new();
//
//     loop {
//         let operation = cpu.step_with_deassembler();
//         operations.push(operation.into());
//         steps += 1;
//
//         if steps >= MAX_STEPS {
//             break;
//         }
//     }
//
//     println!("\n--- MCS-8 BIOS OUTPUT END ---");
//     println!("Executed {} steps", steps);
//     for op in &operations {
//         println!("{}", op);
//     }
//     assert!(false);
// }