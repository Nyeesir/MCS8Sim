pub fn assemble (data: &str){
    const MEMORY_SIZE: usize = 65536;
    let _memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    let _pointer: u16 = 0;

    let lines = data.lines();
    for line in lines{
        if line.is_empty() {continue}

        let mut tokens_iter = line.trim().split_whitespace();

        let token = tokens_iter.next().expect("Non-empty line doesn't contain a word somehow");
        if token.contains(":") {
            handle_label(token);
            let token = tokens_iter.next();
            match token {
                Some(token) => encode_instruction(token),
                None => (continue)
            }
        }
        else {
            encode_instruction(token)
        }
    }
}

fn encode_instruction(instruction: &str){
    match instruction {
        "ADI" => {

        }
        _ => println!("unknown command")
    }
}

fn handle_label(label: &str){

}