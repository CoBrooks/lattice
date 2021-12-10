use super::{ Error, Token };

pub fn simulate(tokens: &Vec<Token>) -> Result<(), Error> {
    // let mem_addr: (usize, usize) = (0, 0);
    // let mem: HashMap<(usize, usize), u8> = HashMap::new();
    let mut stack: Vec<u8> = Vec::new();

    for token in tokens {
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
        }
    }

    Ok(())
}
