use std::{fs, u8};
use std::path::Path;
use std::process::Command;

use super::{ Error, Token, TokenPos, LexerOutput };

#[derive(Debug)]
struct CompilerVars {
    block_num: usize,
    block_addrs: Vec<usize>,
    depth: u8,
    block_is_dowhile: Vec<bool>,
    inside_fn: bool
}

pub fn compile(tokens: &LexerOutput, input_filename: &str) -> Result<(), Error> {
    // struct deconstruction
    let LexerOutput { fn_tokens, tokens } = tokens;

    let mut instructions: Vec<String> = Vec::new();

    let mut compiler_vars = CompilerVars {
        block_num: 0,
        block_addrs: Vec::new(),
        depth: 0,
        block_is_dowhile: Vec::new(),
        inside_fn: false
    };

    instructions.push("section .bss".into());
    // Allocate memory table (array of 32 pointers)
    instructions.push("    mem_table resq 32".into());
    instructions.push("    mem_loc   resq 1".into());
    // Allocate function stack (array of 1024 pointers)
    instructions.push("    fn_stack resq 1024".into());
    instructions.push("    fn_index resq 1".into());

    instructions.push("section .text".into());

    // Import memory functions 
    instructions.push("extern insert_val".into());
    instructions.push("extern get_val".into());
    instructions.push("extern pop_element".into());
    instructions.push("extern init_table".into());
    instructions.push("extern free_table".into());
    instructions.push("extern write_cells".into());
    
    // Function for printing (32-bit) numbers
    instructions.push("print:".into());
    instructions.push("    mov     r9, -3689348814741910323".into());
    instructions.push("    sub     rsp, 40".into());
    instructions.push("    mov     BYTE [rsp+31], 10".into());
    instructions.push("    lea     rcx, [rsp+30]".into());
    instructions.push(".L2:".into());
    instructions.push("    mov     rax, rdi".into());
    instructions.push("    lea     r8, [rsp+32]".into());
    instructions.push("    mul     r9".into());
    instructions.push("    mov     rax, rdi".into());
    instructions.push("    sub     r8, rcx".into());
    instructions.push("    shr     rdx, 3".into());
    instructions.push("    lea     rsi, [rdx+rdx*4]".into());
    instructions.push("    add     rsi, rsi".into());
    instructions.push("    sub     rax, rsi".into());
    instructions.push("    add     eax, 48".into());
    instructions.push("    mov     BYTE [rcx], al".into());
    instructions.push("    mov     rax, rdi".into());
    instructions.push("    mov     rdi, rdx".into());
    instructions.push("    mov     rdx, rcx".into());
    instructions.push("    sub     rcx, 1".into());
    instructions.push("    cmp     rax, 9".into());
    instructions.push("    ja      .L2".into());
    instructions.push("    lea     rax, [rsp+32]".into());
    instructions.push("    mov     edi, 1".into());
    instructions.push("    sub     rdx, rax".into());
    instructions.push("    xor     eax, eax".into());
    instructions.push("    lea     rsi, [rsp+32+rdx]".into());
    instructions.push("    mov     rdx, r8".into());
    instructions.push("    mov     rax, 1".into());
    instructions.push("    syscall".into());
    instructions.push("    add     rsp, 40".into());
    instructions.push("    ret".into());

    // Write function instructions
    for (token, _) in fn_tokens {
        // last instruction in function
        if let Token::End(-1) = token {
            // push fn_stack[fn_index] to stack
            instructions.push("    mov     rcx, QWORD fn_stack".into());
            instructions.push("    mov     rdx, QWORD [fn_index]".into());
            instructions.push("    mov     rax, QWORD [rcx+rdx*8]".into());
            instructions.push("    push    rax".into());
            // decrement fn_index
            instructions.push("    dec     QWORD [fn_index]".into());
        }

        push_instructions_from_token(token, &mut instructions, &mut compiler_vars);

        // first instruction in function
        if let Token::Fn(..) = token {
            // increment fn_index
            instructions.push("    inc     QWORD [fn_index]".into());
            // pop top element of stack into fn_stack[fn_index] 
            instructions.push("    mov     rcx, QWORD fn_stack".into());
            instructions.push("    mov     rdx, QWORD [fn_index]".into());
            instructions.push("    pop     rax".into());
            instructions.push("    mov     QWORD [rcx+rdx*8], rax".into());
        }
    }

    // Executable entry point
    instructions.push("global _start".into());
    instructions.push("_start:".into());

    // Initialize mem_table
    instructions.push("    mov    rdi, mem_table".into());
    instructions.push("    call   init_table".into());
    // Initialize fn_stack
    instructions.push("    mov    QWORD [fn_index], 0".into());

    // Write main function instructions
    for (token, _) in tokens {
        push_instructions_from_token(token, &mut instructions, &mut compiler_vars);
    }

    instructions.push("; -- exit --".into());
    instructions.push("    mov    rdi, mem_table".into());
    instructions.push("    call   free_table".into());
    instructions.push("    mov    rax, 60".into());
    instructions.push("    pop    rdi".into()); // return code = top element on stack
    instructions.push("    syscall".into());

    let file_contents: String = instructions.join("\n");
 
    let output_base = Path::new(input_filename);

    fs::write(output_base.with_extension("asm"), &file_contents).map_err(
        |err| Error { msg: err.to_string(), pos: TokenPos::default() }
    ).expect("Failed to write to file.");

    // Compile asm
    Command::new("nasm")
        .args([
              "-felf64", 
              output_base.with_extension("asm").to_str().unwrap()
        ]).output().expect("Failed to compile assembly.");

    // Compile c lib(s)
    Command::new("gcc")
        .args([
              "-c",
              "-g",
              "-o",
              "mem.o",
              "./src/com/libs/mem.c",
              "-static",
              "-static-libgcc",
        ]).output().expect("Failed to compile c libs.");

    // Link libs to asm
    Command::new("ld")
        .args([
              "-dynamic-linker", 
              "/lib64/ld-linux-x86-64.so.2",
              "-o",
              output_base.with_extension("").to_str().unwrap(),
              "-lc",
              output_base.with_extension("o").to_str().unwrap(),
              "mem.o"
        ]).output().expect("Failed to link.");

    Ok(())
}

fn push_instructions_from_token(token: &Token, instructions: &mut Vec<String>, compiler_vars: &mut CompilerVars) {
    instructions.push(token.to_asm_comment());

    match token {
        Token::Num(num) => {
            instructions.push(format!("    push {}", num));
        },
        Token::OpAdd => {
            instructions.push("    pop    rax".into());
            instructions.push("    pop    rcx".into());
            instructions.push("    add    rax, rcx".into());
            instructions.push("    push   rax".into());
        },
        Token::OpSub => {
            instructions.push("    pop    rcx".into());
            instructions.push("    pop    rax".into());
            instructions.push("    sub    rax, rcx".into());
            instructions.push("    push   rax".into());
        },
        Token::OpMul => {
            instructions.push("    pop    rax".into());
            instructions.push("    pop    rcx".into());
            instructions.push("    mul    rcx".into());
            instructions.push("    push   rax".into());
        },
        Token::OpDiv => {
            instructions.push("    pop    rcx".into());
            instructions.push("    pop    rax".into());
            // div performs division against the 64 bit number rdx:rax,
            // so rdx needs to be zero'd before the division
            instructions.push("    xor    rdx, rdx".into()); 
            instructions.push("    div    rcx".into()); 
            instructions.push("    push   rax".into());
        },
        Token::Print => {
            instructions.push("    pop    rdi".into());
            instructions.push("    call   print".into());
        },
        Token::Write => {
            instructions.push("    mov    rdi, mem_table".into());
            instructions.push("    mov    rsi, [mem_loc]".into());
            instructions.push("    pop    rdx".into());
            instructions.push("    call   write_cells".into());
        },
        Token::Dup => {
            instructions.push("    pop    rax".into());
            instructions.push("    push   rax".into());
            instructions.push("    push   rax".into());
        },
        Token::Drop => {
            instructions.push("    pop    rax".into());
        },
        Token::Swap => {
            instructions.push("    pop    rax".into());
            instructions.push("    pop    rcx".into());
            instructions.push("    push   rax".into());
            instructions.push("    push   rcx".into());
        },
        Token::Over => {
            instructions.push("    pop    rax".into());
            instructions.push("    pop    rcx".into());
            instructions.push("    push   rcx".into());
            instructions.push("    push   rax".into());
            instructions.push("    push   rcx".into());
        },
        Token::If(_) => {
            instructions.push("    pop    rax".into());
            instructions.push("    cmp    rax, 0".into());
            instructions.push(format!("    je     addr_{}", compiler_vars.block_num));
            compiler_vars.block_addrs.push(compiler_vars.block_num);
            compiler_vars.block_num += 1;
            compiler_vars.depth += 1;
            compiler_vars.block_is_dowhile.push(false);
        },
        Token::Else(_) => {
            let block_addr = compiler_vars.block_addrs.pop().unwrap();
            instructions.push(format!("    jmp    addr_{}", compiler_vars.block_num));
            compiler_vars.block_addrs.push(compiler_vars.block_num);
            compiler_vars.block_num += 1;
            instructions.push(format!("addr_{}:", block_addr));
        },
        Token::While => {
            instructions.push(format!("addr_{}:", compiler_vars.block_num));
            compiler_vars.block_num += 1;
            compiler_vars.block_addrs.push(compiler_vars.block_num);
            compiler_vars.block_addrs.push(compiler_vars.block_num - 1);
            compiler_vars.block_num += 1;
            compiler_vars.depth += 1;
        },
        Token::Do(_) => {
            instructions.push("    pop    rax".into());
            instructions.push("    cmp    rax, 0".into());
            instructions.push(format!("    je     addr_{}", compiler_vars.block_addrs[compiler_vars.block_addrs.len() - 2]));
            compiler_vars.block_is_dowhile.push(true);
        },
        Token::End(ip) => {
            if *ip == -1isize {
                instructions.push("    ret".into());
                compiler_vars.inside_fn = false;
            } else {
                if compiler_vars.block_is_dowhile.pop().is_some() {
                    instructions.push(format!("    jmp    addr_{}", compiler_vars.block_addrs.pop().unwrap()));
                }

                instructions.push(format!("addr_{}:", compiler_vars.block_addrs.pop().unwrap()));
            }

            compiler_vars.depth -= 1;
        },
        Token::Eq | Token::And => {
            instructions.push("    pop    rcx".into());
            instructions.push("    pop    rdx".into());
            instructions.push("    cmp    rdx, rcx".into());
            instructions.push("    mov    rax, 0".into());
            instructions.push("    mov    rdx, 1".into());
            instructions.push("    cmove  rax, rdx".into());
            instructions.push("    push   rax".into());
        },
        Token::GT => {
            instructions.push("    pop    rcx".into());
            instructions.push("    pop    rdx".into());
            instructions.push("    cmp    rdx, rcx".into());
            instructions.push("    mov    rax, 0".into());
            instructions.push("    mov    rdx, 1".into());
            instructions.push("    cmovg  rax, rdx".into());
            instructions.push("    push   rax".into());
        },
        Token::LT => {
            instructions.push("    pop    rcx".into());
            instructions.push("    pop    rdx".into());
            instructions.push("    cmp    rdx, rcx".into());
            instructions.push("    mov    rax, 0".into());
            instructions.push("    mov    rdx, 1".into());
            instructions.push("    cmovl  rax, rdx".into());
            instructions.push("    push   rax".into());
        },
        Token::Not => {
            instructions.push("    pop    rax".into());
            instructions.push("    cmp    rax, 0".into());
            instructions.push("    mov    rcx, 1".into());
            instructions.push("    cmove  rax, rcx".into());
            instructions.push("    push   rax".into());
        },
        Token::Or => {
            instructions.push("    pop    rax".into());
            instructions.push("    pop    rcx".into());
            instructions.push("    mov    rdx, 1".into());
            instructions.push("    cmp    rax, 0".into());
            instructions.push("    cmovne rax, rdx".into());
            instructions.push("    cmp    rcx, 0".into());
            instructions.push("    cmovne rax, rdx".into());
            instructions.push("    push   rax".into());
        },
        Token::Up => {
            instructions.push("    pop    rax".into());
            instructions.push("    mov    rcx, 4294967296".into());
            instructions.push("    mul    rcx".into());
            instructions.push("    sub    [mem_loc], rax".into());
        },
        Token::Down => {
            instructions.push("    pop    rax".into());
            instructions.push("    mov    rcx, 4294967296".into());
            instructions.push("    mul    rcx".into());
            instructions.push("    add    [mem_loc], rax".into());
        },
        Token::Left => {
            instructions.push("    pop    rax".into());
            instructions.push("    sub    [mem_loc], rax".into());
        },
        Token::Right => {
            instructions.push("    pop    rax".into());
            instructions.push("    add    [mem_loc], rax".into());
        },
        Token::Loc => {
            instructions.push("    mov    rax, [mem_loc]".into());
            instructions.push("    push   rax".into());
        },
        Token::Store => {
            instructions.push("    mov    rdi, mem_table".into());
            instructions.push("    mov    rsi, [mem_loc]".into());
            instructions.push("    pop    rdx".into());
            instructions.push("    call   insert_val".into());
        },
        Token::Load => {
            instructions.push("    mov    rdi, mem_table".into());
            instructions.push("    mov    rsi, [mem_loc]".into());
            instructions.push("    xor    rax, rax".into());
            instructions.push("    call   pop_element".into());
            instructions.push("    push   rax".into());
        },
        Token::Copy => {
            instructions.push("    mov    rdi, mem_table".into());
            instructions.push("    mov    rsi, [mem_loc]".into());
            instructions.push("    xor    rax, rax".into());
            instructions.push("    call   get_val".into());
            instructions.push("    push   rax".into());
        },
        Token::Fn(name, _) => {
            let name = String::from_utf8_lossy(name);
            let (name, _) = name.split_once('\0').unwrap();

            instructions.push(format!("{}: ", name));
            compiler_vars.depth += 1;
            compiler_vars.inside_fn = true;
        }
        Token::FnCall(name) => {
            let name = String::from_utf8_lossy(name);
            let (name, _) = name.split_once('\0').unwrap();
            
            instructions.push(format!("    call {}", name));
        }
    }
}
