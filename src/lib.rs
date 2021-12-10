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
pub enum Error {
    FileLoadError(String), 
    SyntaxError(String)
}
impl std::error::Error for Error { }
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::FileLoadError(msg) => write!(f, "ERROR: {}", msg),
            Error::SyntaxError(msg) => write!(f, "ERROR: {}", msg),
        }
    }
}

pub fn load_file(filepath: &str) -> Result<Vec<String>, Error> {
    use std::fs;

    if let Ok(file) = fs::read_to_string(filepath) {
        Ok(file.lines().map(|l| l.to_string()).collect())
    } else {
        Err(Error::FileLoadError(format!("Unable to open file: {}", filepath)))
    }
}

#[derive(Debug)]
pub enum Token {
    Num(u8),
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    Print,
    Dup,
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
        }.into()
    }
}

pub fn lex_lines(lines: Vec<String>) -> Result<Vec<Token>, Error> {
    let mut tokens: Vec<Token> = Vec::new();

    for line in lines {
        let ts: Vec<&str> = line.split_ascii_whitespace().collect();

        for t in ts {
            match t {
                t if t.chars().all(|c| c.is_ascii_digit()) => {
                    if let Ok(num) = t.parse::<u8>() {
                        tokens.push(Token::Num(num))
                    } else {
                        return Err(Error::SyntaxError(
                            format!("Invalid number {}. Numbers must be non-negative and less than 256.", t)
                        ));
                    }
                },
                "+" => tokens.push(Token::OpAdd),
                "-" => tokens.push(Token::OpSub),
                "*" => tokens.push(Token::OpMul),
                "/" => tokens.push(Token::OpDiv),
                "print" => tokens.push(Token::Print),
                "dup" => tokens.push(Token::Dup),
                _ => panic!("Unimplemented token {}", t)
            }
        }
    }

    Ok(tokens)
}
