use super::{ Error, Token, TokenPos };

pub fn simulate(tokens: &Vec<(Token, TokenPos)>) -> Result<(), Error> {
    // let mem_addr: (usize, usize) = (0, 0);
    // let mem: HashMap<(usize, usize), u8> = HashMap::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut ip: usize = 0;

    while ip < tokens.len() {
        let (token, _) = &tokens[ip];

        match token {
            Token::Num(num) => {
                stack.push(num.clone());
            },
            Token::OpAdd => {
                let a = stack.pop().expect("Not enough elements on the stack to add.");
                let b = stack.pop().expect("Not enough elements on the stack to add.");
                stack.push(a + b);
            },
            Token::OpSub => {
                let a = stack.pop().expect("Not enough elements on the stack to subtract.");
                let b = stack.pop().expect("Not enough elements on the stack to subtract.");
                stack.push(b - a);
            },
            Token::OpMul => {
                let a = stack.pop().expect("Not enough elements on the stack to multiply.");
                let b = stack.pop().expect("Not enough elements on the stack to multiply.");
                stack.push(a * b);
            },
            Token::OpDiv => {
                let a = stack.pop().expect("Not enough elements on the stack to divide.");
                let b = stack.pop().expect("Not enough elements on the stack to divide.");
                stack.push(b / a);
            },
            Token::Print => {
                let a = stack.pop().expect("Not enough elements on the stack to print.");
                println!("{}", a);
            },
            Token::Dup => {
                let a = stack.pop().expect("No element to duplicate.");
                stack.push(a);
                stack.push(a);
            },
            Token::Drop => {
                let _ = stack.pop().expect("No element to drop.");
            },
            Token::Swap => {
                let a = stack.pop().expect("Not enough elements to swap.");
                let b = stack.pop().expect("Not enough elements to swap.");
                stack.push(a);
                stack.push(b);
            },
            Token::Over => {
                let a = stack.pop().expect("Not enough elements to duplicate over.");
                let b = stack.pop().expect("Not enough elements to duplicate over.");
                stack.push(b);
                stack.push(a);
                stack.push(b);
            },
            Token::If(next_ip) => {
                let a = stack.pop().expect("No element on stack for if block.");
                if a == 0 {
                    ip = *next_ip;
                }
            },
            Token::Else(end_ip) => { 
                ip = *end_ip;
            },
            Token::End(next_ip) => {
                ip = *next_ip;
            },
        }

        ip += 1;
    }

    Ok(())
}
