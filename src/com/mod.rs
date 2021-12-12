use std::fs;
use std::path::Path;
use std::process::Command;

use super::{ Error, Token, TokenPos };

pub fn compile(tokens: &Vec<(Token, TokenPos)>, input_filename: &str, run_on_success: bool) -> Result<(), Error> {
    let mut instructions: Vec<String> = Vec::new();
    
    let mut block_num: usize = 0;
    let mut block_addrs: Vec<usize> = Vec::new();

    let mut block_is_dowhile: Vec<bool> = Vec::new();

    instructions.push("section .bss".into());
    // Allocate memory table (array of 32 pointers)
    instructions.push("    mem_table resb 32".into());
    instructions.push("    mem_loc   resq 1".into());

    instructions.push("section .text".into());

    // Import memory functions 
    instructions.push("extern insert_val".into());
    instructions.push("extern get_val".into());
    instructions.push("extern pop_element".into());
    instructions.push("extern init_table".into());
    instructions.push("extern free_table".into());
    
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

    instructions.push("global _start".into());
    instructions.push("_start:".into());

    // Initialize mem_table
    instructions.push("    mov    rdi, mem_table".into());
    instructions.push("    call   init_table".into());

    for (token, _) in tokens {
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
                instructions.push(format!("    je     addr_{}", block_num));
                block_addrs.push(block_num);
                block_num += 1;
                block_is_dowhile.push(false);
            },
            Token::Else(_) => {
                let block_addr = block_addrs.pop().unwrap();
                instructions.push(format!("    jmp    addr_{}", block_num));
                block_addrs.push(block_num);
                block_num += 1;
                instructions.push(format!("addr_{}:", block_addr));
            },
            Token::While => {
                instructions.push(format!("addr_{}:", block_num));
                block_num += 1;
                block_addrs.push(block_num);
                block_addrs.push(block_num - 1);
                block_num += 1;
            },
            Token::Do(_) => {
                instructions.push("    pop    rax".into());
                instructions.push("    cmp    rax, 0".into());
                instructions.push(format!("    je     addr_{}", block_addrs[block_addrs.len() - 2]));
                block_is_dowhile.push(true);
            },
            Token::End(_) => {
                if block_is_dowhile.pop().unwrap() {
                    instructions.push(format!("    jmp    addr_{}", block_addrs.pop().unwrap()));
                }
                instructions.push(format!("addr_{}:", block_addrs.pop().unwrap()));
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
        }
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

    // Compile c libs
    Command::new("gcc")
        .args([
              "-c", 
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
