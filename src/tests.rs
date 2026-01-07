use super::*;
#[test]
fn assembler_test_1() {
    let data = "MOV B,B \n    add a \n ADD b";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..3], &[64,135,128]);
}
#[test]
fn assembler_test_2() {
    let data = "MOV B,B \njamnik:    add a \n ADD b\n STA jamnik";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..6], &[64,135,128,0x32,0x01,0x00]);
}
#[test]
fn assembler_test_3() {
    let data = "MOV B , B \n    add   a \n ADD    b\nMVI c, 100b ;komentarz\n  ADD c \n MOV B ,C";
    assembler::Assembler::new().assemble(data).unwrap();
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
    let data = "MVI C, 100b ;komentarz";
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
    assembler::Assembler::new().assemble(data).unwrap();
}

#[test]
fn assembler_sta_test_1() {
    let data = "STA 5B12H";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..3], &[0x32,0x12,0x5b]);
}
#[test]
#[should_panic]
fn assembler_sta_test_2() {
    let data = "STA jamnik";
    assembler::Assembler::new().assemble(data).unwrap();
}

#[test]
fn assembler_jmp_test_1(){
    let data = "JMP 1720";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..3], &[0xc3,0xb8,0x06]);
}
#[test]
fn assembler_jnz_test_1() {
    let data = "   jamnik: MOV    B,B\n    add a \n ADD b ;tez\n JNZ   jamnik  ";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..6], &[64,135,128,0xc2,0x00,0x00]);
}
#[test]
fn assembler_jnz_test_2() {
    let data = "   :jamnik: MOV    B,B\n    add a \n ADD b ;tez\n JNZ   :jamnik  ";
    let memory = assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..6], &[64,135,128,0xc2,0x00,0x00]);
}

#[test]
#[should_panic]
fn label_validation_test_1(){
    let data = "świerszcz:";
    assembler::Assembler::new().assemble(data).unwrap();

}

#[test]
#[should_panic]
fn label_validation_test_2(){
    let data = "add:";
    assembler::Assembler::new().assemble(data).unwrap();
}

#[test]
#[should_panic]
fn label_validation_test_3(){
    let data = "1label:";
    assembler::Assembler::new().assemble(data).unwrap();
}

#[test]
fn label_validation_test_4(){
    let data = "label: MOV B,B";
    assembler::Assembler::new().assemble(data).unwrap();
}

#[test]
fn label_validation_test_5(){
    let data = "label: MOV B , B";
    assembler::Assembler::new().assemble(data).unwrap();
}

#[test]
fn assembler_stax_test_1(){
    let data = "STAX B";
    let memory =assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..1], &[0x02]);
}

#[should_panic]
#[test]
fn assembler_lxi_test_1(){
    let data = "LXI B    ,A    , 22354";
    let memory =assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..1], &[0x02]);
}

#[test]
fn assembler_db_test_1(){
    let data = "DB -03h";
    let memory =assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..1], &[0xfd]);
}

#[test]
fn assembler_label_test_1(){
    let data = "JNZ JAMNIK\n MOV B,B \n ADD A \nJAMNIK: DAD B";
    let memory =assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..6], &[0xc2, 0x05, 0x00, 0x40, 0x87, 0x09]);
}

#[should_panic]
#[test]
fn assembler_label_test_2(){
    let data = "JNZ JAMnik\n MOV B,B \n ADD A \nJAMNIK: DAD B";
    let memory =assembler::Assembler::new().assemble(data).unwrap();

    assert_eq!(&memory[0..6], &[0xc2, 0x05, 0x00, 0x40, 0x87, 0x09]);
}
