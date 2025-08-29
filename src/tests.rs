use super::*;
#[test]
fn assembler_test_1() {
    let data = "MOV B,B \n    add a \n ADD z";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..3], &[64,135,128]);
}

