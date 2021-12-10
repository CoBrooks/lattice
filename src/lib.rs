pub mod com;
pub mod sim;

pub fn uncons<T>(arr: &[T]) -> (&T, &[T]){
    if arr.len() > 1 {
        (&arr[0], &arr[1..])
    } else if arr.len() == 1 {
        (&arr[0], &[])
    } else {
        panic!("No elements in array to uncons.")
    }
}

#[derive(Debug)]
pub struct Error {
    msg: String,
    pos: TokenPos
}
impl std::error::Error for Error { }
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ERROR: {}", self.pos, self.msg)
    }
}

pub fn load_file(filepath: &str) -> Result<Vec<String>, Error> {
    use std::fs;

    if let Ok(file) = fs::read_to_string(filepath) {
        Ok(file.lines().map(|l| l.to_string()).collect())
    } else {
        Err(Error { 
            msg: format!("Unable to open file: {}", filepath), 
            pos: TokenPos::default() 
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Token {
    Num(usize),
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    Print,
    Dup,
    Drop,
    Swap,
    Over,
    If(usize),
    Else(usize),
    End(usize),
}

impl Token {
    pub fn to_asm_comment(&self) -> String {
        match self {
            Token::Num(_) => "; -- push --",
            Token::OpAdd => "; -- add --",
            Token::OpSub => "; -- sub --",
            Token::OpMul => "; -- mul --",
            Token::OpDiv => "; -- div --",
            Token::Print => "; -- print --",
            Token::Dup => "; -- dup --",
            Token::Drop => "; -- drop --",
            Token::Swap => "; -- swap --",
            Token::Over => "; -- over --",
            Token::If(_) => "; -- if --",
            Token::Else(_) => "; -- else --",
            Token::End(_) => "; -- end --",
        }.into()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TokenPos {
    pub row: usize,
    pub col: usize,
    pub ip: usize
}
impl std::fmt::Display for TokenPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.col)
    }
}

pub fn get_block_end(tokens: &[(Token, TokenPos)], block_start: TokenPos) -> Result<usize, Error> {
    let mut ip = block_start.ip + 1;
    let mut block_depth = 1;

    while ip < tokens.len() {
        let (token, _) = &tokens[ip];
        match token {
            Token::If(_) => block_depth += 1,
            Token::End(_) => block_depth -= 1,
            _ => { }
        }

        if block_depth == 0 {
            return Ok(ip);
        }

        ip += 1;
    }

    Err(Error { msg: "Block is not terminated with the `end` keyword.".into(), pos: block_start })
}

pub fn get_if_next_ip(tokens: &[(Token, TokenPos)], if_start: TokenPos) -> Result<usize, Error> {
    let mut ip = if_start.ip + 1;
    let mut block_depth = 1;

    while ip < tokens.len() {
        let (token, _) = &tokens[ip];
        match token {
            Token::If(_) => block_depth += 1,
            Token::Else(_) => {
                if block_depth == 1 {
                    return Ok(ip);
                }            
            },
            Token::End(_) => block_depth -= 1,
            _ => { }
        }

        if block_depth == 0 {
            return Ok(ip)
        }

        ip += 1;
    }

    Err(Error { msg: "If statement is not terminated with the `end` keyword".into(), pos: if_start })
}

pub fn lex_lines(lines: Vec<String>) -> Result<Vec<(Token, TokenPos)>, Error> {
    let mut tokens: Vec<(Token, TokenPos)> = Vec::new();
    let mut ip: usize = 0;

    let mut blocks: Vec<(Token, TokenPos)> = Vec::new();

    for (row, line) in lines.iter().enumerate() {
        let ts: Vec<&str> = line.split_ascii_whitespace().collect();

        for (col, t) in ts.iter().cloned().enumerate() {
            let pos = TokenPos { 
                row,
                col,
                ip
            };

            let token = match t {
                t if t.chars().all(|c| c.is_ascii_digit()) => {
                    if let Ok(num) = t.parse::<usize>() {
                        Token::Num(num)
                    } else {
                        return Err(Error {
                            msg: format!("Invalid number {}.", t),
                            pos
                        });
                    }
                },
                "+" => Token::OpAdd,
                "-" => Token::OpSub,
                "*" => Token::OpMul,
                "/" => Token::OpDiv,
                "print" => Token::Print,
                "dup" => Token::Dup,
                "drop" => Token::Drop,
                "swap" => Token::Swap,
                "over" => Token::Over,
                "if" => { 
                    let t = Token::If(0);
                    blocks.push((t, pos));
                    t
                },
                "else" => {
                    let t = Token::Else(0);
                    blocks.push((t, pos));
                    t
                },
                "end" => {
                    let (block_type, block_start) = blocks.last().expect("End must have a corresponding block.");
                    if let Token::If(_) = block_type {
                        Token::End(ip)
                    } else if let Token::Else(_) = block_type { 
                        Token::End(ip)
                    } else {
                        Token::End(block_start.ip) 
                    }
                },
                _ => { 
                    return Err(Error {
                        msg: format!("Unimplemented token {}", t),
                        pos
                    })
                },
            };

            tokens.push((token, pos));
            ip += 1;
        }
    }

    for (_, block_start) in blocks {
        if let (Token::If(_), ip) = tokens[block_start.ip] {
            let next = get_if_next_ip(tokens.as_slice(), block_start)?;
            
            tokens[block_start.ip] = (Token::If(next), ip);
        } else if let (Token::Else(_), ip) = tokens[block_start.ip] { 
            let next = get_block_end(tokens.as_slice(), block_start)?;

            tokens[block_start.ip] = (Token::Else(next), ip);
        } else {
            todo!("Implement missing block logic.")
        }
    }

    Ok(tokens)
}
