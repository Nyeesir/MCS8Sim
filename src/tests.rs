use super::*;
#[test]
fn assembler_test_1() {
    let data = "ADI 30H \n\r
                        RST 1 \n\r
                        DCX H \n\r
                        DCR B \n\r" ;
    assembler::assemble(data);
    assert!(false);
}

