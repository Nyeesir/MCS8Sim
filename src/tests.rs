use super::*;
#[test]
fn assembler_test_1() {
    let data = "MOV B,B \n    add a \n ADD b";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..3], &[64,135,128]);
}

#[test]
fn assembler_mvi_test_1() {
    let data = "MVI B,'a'";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..2], &[0x06,'a' as u8]);
}
#[test]
fn assembler_mvi_test_2() {
    let data = "MVI B,137";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..2], &[0x06,137]);
}
#[test]
fn assembler_mvi_test_3() {
    let data = "MVI C,100b";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..2], &[0x0e,0b100]);
}
#[test]
fn assembler_mvi_test_4() {
    let data = "MVI C,70q";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..2], &[0x0e,0o70]);
}
#[test]
#[should_panic]
fn assembler_mvi_test_5() {
    let data = "MVI B,256";
    let memory = assembler::Assembler::new().assemble(data).unwrap();
}

#[test]
#[should_panic]
fn label_validation_test_1(){
    let data = "świerszcz:";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

}

#[test]
#[should_panic]
fn label_validation_test_2(){
    let data = "add:";
    let memory = assembler::Assembler::new().assemble(data).unwrap();
}

#[test]
#[should_panic]
fn label_validation_test_3(){
    let data = "1label:";
    let memory = assembler::Assembler::new().assemble(data).unwrap();
}

#[test]
fn label_validation_test_4(){
    let data = "label: MOV B,B";
    let memory = assembler::Assembler::new().assemble(data).unwrap();
}