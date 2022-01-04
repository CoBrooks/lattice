use std::collections::{ HashMap, HashSet };

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

fn byte_slice_to_fixed_len<const N: usize>(s: &[u8], fill: u8) -> [u8; N] {
    assert!(s.len() <= N);

    let mut a = [fill; N];

    for (i, elem) in s.iter().enumerate() {
        a[i] = elem.clone();
    }

    a
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
        let mut lines: Vec<String> = file.lines()
           .map(|l| 
                l.split("//").find(|_| true).unwrap().to_string()
            ).collect();

        let mut const_dict: HashMap<String, usize> = HashMap::new();
        let const_declarations: Vec<String> = lines.iter().filter(|l| l.starts_with("#const")).cloned().collect();
        lines.retain(|l| !l.starts_with("#const"));

        if !const_declarations.is_empty() {
            for decl in const_declarations {
                if let [_, name, val] = decl.split(' ').collect::<Vec<&str>>().as_slice() {
                    const_dict.insert(name.to_string(), val.parse().expect("Unable to parse constant value."));
                } else {
                    return Err(Error { 
                        msg: format!("Unable to parse const declaration: '{}'", decl), 
                        pos: TokenPos::default() 
                    });
                }
            }

            for line in lines.iter_mut() {
                let line_clone = line.clone();
                let words: Vec<&str> = line_clone.split(' ').collect();

                for word in words {
                    if const_dict.contains_key(word.trim()) {
                        *line = line.replace(word, &const_dict.get(word.trim()).unwrap().to_string());
                    }
                }
            }
        }

        Ok(lines)
    } else {
        Err(Error { 
            msg: format!("Unable to open file: {}", filepath), 
            pos: TokenPos::default() 
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Token {
    // For pushing a number to the stack
    Num(usize),

    // Mathematical operations
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,

    // Stdout interaction
    Print,
    Write,
    
    // Stack operations
    Dup,
    Drop,
    Swap,
    Over,

    // Conditionals and Loops
    If(usize),
    Else(usize),
    While,
    Do(usize),
    End(isize),

    // Conditional operators
    Eq,
    GT,
    LT,
    And,
    Not,
    Or,

    // Memory interaction
    Up,
    Down,
    Left,
    Right,
    Loc,
    Store,
    Load,
    Copy,

    // Functions
    Fn([u8; 256], usize),
    FnCall([u8; 256]),
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
            Token::Write => "; -- write --",
            Token::Dup => "; -- dup --",
            Token::Drop => "; -- drop --",
            Token::Swap => "; -- swap --",
            Token::Over => "; -- over --",
            Token::If(_) => "; -- if --",
            Token::Else(_) => "; -- else --",
            Token::While => "; -- while --",
            Token::Do(_) => "; -- do --",
            Token::End(_) => "; -- end --",
            Token::Eq => "; -- eq --",
            Token::GT => "; -- gt --",
            Token::LT => "; -- lt --",
            Token::And => "; -- and --",
            Token::Not => "; -- not --",
            Token::Or => "; -- or --",
            Token::Up => "; -- up --",
            Token::Down => "; -- down --",
            Token::Left => "; -- left --",
            Token::Right => "; -- right --",
            Token::Loc => "; -- loc --",
            Token::Store => "; -- store --",
            Token::Load => "; -- load --",
            Token::Copy => "; -- copy --",
            Token::Fn(_, _) => "; -- fn --",
            Token::FnCall(_) => "; -- fn call --",
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
            Token::If(_) | Token::Do(_) => block_depth += 1,
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
            Token::If(_) | Token::Do(_) => block_depth += 1,
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

pub struct LexerOutput {
    pub tokens: Vec<(Token, TokenPos)>,
    pub fn_tokens: Vec<(Token, TokenPos)>,
}

pub fn lex_lines(lines: Vec<String>) -> Result<LexerOutput, Error> {
    let mut tokens: Vec<(Token, TokenPos)> = Vec::new();
    let mut fn_tokens: Vec<(Token, TokenPos)> = Vec::new();
    let mut ip: usize = 0;

    let mut blocks: Vec<(Token, TokenPos)> = Vec::new();
    let mut terminated_blocks: Vec<(Token, TokenPos)> = Vec::new();

    let mut functions: HashSet<&[u8]> = HashSet::new();
    let mut inside_fn: bool = false;

    for (row, line) in lines.iter().enumerate() {
        let ts: Vec<&str> = line.split_ascii_whitespace().collect();

        let mut prev_fn = false;

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
                "=" => Token::Eq,
                ">" => Token::GT,
                "<" => Token::LT,
                "and" => Token::And,
                "not" => Token::Not,
                "or" => Token::Or,
                "print" => Token::Print,
                "write" => Token::Write,
                "dup" => Token::Dup,
                "drop" => Token::Drop,
                "swap" => Token::Swap,
                "over" => Token::Over,
                "u" => Token::Up,
                "d" => Token::Down,
                "l" => Token::Left,
                "r" => Token::Right,
                "loc" => Token::Loc,
                "." => Token::Store,
                "," => Token::Load,
                "?" => Token::Copy,
                "if" => { 
                    let t = Token::If(0);
                    blocks.push((t, pos));
                    terminated_blocks.push((t, pos));
                    t
                },
                "else" => {
                    let t = Token::Else(0);
                    if let (Token::If(_), _) = terminated_blocks.last().unwrap_or(&(Token::Not, TokenPos::default())) {
                        terminated_blocks.pop();
                    }
                    blocks.push((t, pos));
                    terminated_blocks.push((t, pos));
                    t
                },
                "while" => { 
                    let t = Token::While;
                    blocks.push((t, pos));
                    terminated_blocks.push((t, pos));
                    t
                },
                "do" => { 
                    let t = Token::Do(0);
                    blocks.push((t, pos));
                    t
                },
                "end" => {
                    let (block_type, block_start) = terminated_blocks.pop().unwrap();
                    if let Token::If(_) = block_type {
                        Token::End(ip as isize)
                    } else if let Token::Else(_) = block_type { 
                        Token::End(ip as isize)
                    } else if let Token::While = block_type { 
                        Token::End(block_start.ip as isize)
                    } else if let Token::Fn(_, _) = block_type { 
                        inside_fn = false;
                        Token::End(-1)
                    } else {
                        Token::End(block_start.ip as isize) 
                    }
                },
                "fn" => {
                    let t = Token::Fn([0; 256], 0);

                    prev_fn = true;

                    t
                },
                t if prev_fn => {
                    let (_, pos) = tokens.pop().unwrap();
                    let mut fn_name = [0; 256];
                    let chars = t.as_bytes();

                    assert!(chars.len() <= 256);

                    for (i, c) in chars.into_iter().enumerate() {
                        fn_name[i] = *c;
                    }

                    prev_fn = false;
                    functions.insert(t.as_bytes());
                    inside_fn = true;

                    let t = Token::Fn(fn_name, 0);
                    terminated_blocks.push((t, pos));

                    ip -= 1;

                    t
                },
                t if functions.contains(t.as_bytes()) => {
                    if let Some(fn_name) = functions.get(t.as_bytes()) {
                        Token::FnCall(byte_slice_to_fixed_len(fn_name, 0))
                    } else {
                        panic!("Function name not found")
                    }
                },
                _ => { 
                    return Err(Error {
                        msg: format!("Unimplemented token {}", t),
                        pos
                    })
                },
            };

            if inside_fn {
                fn_tokens.push((token, pos));
            } else if let Token::End(-1) = token {
                fn_tokens.push((token, pos));
            } else {
                tokens.push((token, pos));
            }
            ip += 1;
        }
    }

    for (b, block_start) in &blocks {
        if let (Token::If(_), ip) = tokens[block_start.ip] {
            let next = get_if_next_ip(tokens.as_slice(), *block_start)?;
            
            tokens[block_start.ip] = (Token::If(next), ip);
        } else if let (Token::Else(_), ip) = tokens[block_start.ip] { 
            let next = get_block_end(tokens.as_slice(), *block_start)?;

            tokens[block_start.ip] = (Token::Else(next), ip);
        } else if let (Token::Do(_), ip) = tokens[block_start.ip] {
            let end_ip = get_block_end(tokens.as_slice(), *block_start)?;

            tokens[block_start.ip] = (Token::Do(end_ip), ip);
        } else if let (Token::While, _) = tokens[block_start.ip] { 
            
        } else if let (Token::Fn(fn_name, _), ip) = (b, block_start) {
            // let end_ip = get_block_end(fn_tokens.as_slice(), *block_start)?;

            // tokens[block_start.ip] = (Token::Fn(*fn_name, end_ip), *ip);
        } else {
            println!("BLOCKS: {:?} \n B {:?} \n BLOCK START {:?}", &blocks, b, tokens[block_start.ip]);
            todo!("Implement missing block logic.")
        }
    }
    
    // #[cfg(debug_assertions)]
    // for (token, pos) in &tokens {
    //     println!("{:?}: {}", token, pos.ip);
    // }

    Ok(LexerOutput {
        tokens,
        fn_tokens
    })
}
