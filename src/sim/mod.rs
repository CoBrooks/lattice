use std::collections::BTreeMap;

use super::{ Error, Token, TokenPos };

pub fn simulate(tokens: &Vec<(Token, TokenPos)>) -> Result<(), Error> {
    let mut mem_addr: (u32, u32, u64) = (0, 0, 0);
    let mut mem: BTreeMap<u64, usize> = BTreeMap::new();

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
            Token::While => { },
            Token::Do(end_ip) => { 
                let a = stack.pop().expect("Do-while loop must have value on the stack for the condition.");
                if a == 0 {
                    ip = *end_ip;
                }
            },
            Token::End(next_ip) => {
                ip = *next_ip;
            },
            Token::Eq => {
                let a = stack.pop().expect("No element on stack to compare.");
                let b = stack.pop().expect("No element on stack to compare.");
                stack.push((a == b) as usize);
            },
            Token::GT => {
                let a = stack.pop().expect("No element on stack to compare.");
                let b = stack.pop().expect("No element on stack to compare.");
                stack.push((b > a) as usize);
            },
            Token::LT => {
                let a = stack.pop().expect("No element on stack to compare.");
                let b = stack.pop().expect("No element on stack to compare.");
                stack.push((b < a) as usize);
            },
            Token::And => {
                let a = stack.pop().expect("No element on stack to compare.");
                let b = stack.pop().expect("No element on stack to compare.");
                stack.push((a > 0 && b > 0) as usize);
            },
            Token::Not => {
                let a = stack.pop().expect("No element on stack to compare.");
                stack.push((a == 0) as usize);
            },
            Token::Or => {
                let a = stack.pop().expect("No element on stack to compare.");
                let b = stack.pop().expect("No element on stack to compare.");
                stack.push((a > 0 || b > 0) as usize);
            },
            Token::Up => {
                let a = stack.pop().expect("Up requires a magnitude to traverse the grid.");
                let (x, y, ptr) = mem_addr;
                mem_addr = (x, y - a as u32, ptr - (a as u64 * u32::MAX as u64));
            },
            Token::Down => {
                let a = stack.pop().expect("Down requires a magnitude to traverse the grid.");
                let (x, y, ptr) = mem_addr;
                mem_addr = (x, y + a as u32, ptr + (a as u64 * u32::MAX as u64));
            },
            Token::Left => {
                let a = stack.pop().expect("Left requires a magnitude to traverse the grid.");
                let (x, y, ptr) = mem_addr;
                mem_addr = (x - a as u32, y, ptr - a as u64);
            },
            Token::Right => {
                let a = stack.pop().expect("Right requires a magnitude to traverse the grid.");
                let (x, y, ptr) = mem_addr;
                mem_addr = (x + a as u32, y, ptr + a as u64);
            },
            Token::Loc => {
                let (_, _, ptr) = mem_addr;
                stack.push(ptr as usize);
            },
            Token::Store => {
                let a = stack.pop().expect("There must be a value on the stack to store.");
                let (_, _, ptr) = mem_addr;
                mem.insert(ptr, a);
            },
            Token::Load => {
                let (_, _, ptr) = mem_addr;
                if let Some(a) = mem.get_mut(&ptr) {
                    stack.push(a.clone());
                    *a = 0;
                } else {
                    stack.push(0);
                }
            },
            Token::Copy => {
                let (_, _, ptr) = mem_addr;
                if let Some(a) = mem.get(&ptr) {
                    stack.push(*a);
                } else {
                    stack.push(0);
                }
            },
        }

        ip += 1;
    }

    Ok(())
}
