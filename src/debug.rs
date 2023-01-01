use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, EditMode, Editor};
use std::process;

use crate::cpu::CPU;
use crate::cpu::Opcode;

#[derive(Debug)]
pub enum DebuggerRet {
    Step,
    Continue,
}

pub fn print_registers(cpu: &CPU) {
    println!("A: {:#04x}, B: {:#04x}, C: {:#04x}, D: {:#04x}, E: {:#04x}, H: {:#04x}, L: {:#04x}, PC: {:#04x}, SP: {:#04x}",
             cpu.reg.a, cpu.reg.b, cpu.reg.c, cpu.reg.d, cpu.reg.e, cpu.reg.h, cpu.reg.l, cpu.reg.pc, cpu.reg.sp)
}


pub fn list_assembly(cpu: &CPU) {
    for a in (cpu.reg.pc-6)..(cpu.reg.pc+6) {
        let b = cpu.memory_bus.read_byte(a);
        if a == cpu.reg.pc {
            print!("->");
        }
        print!("\t{:#04x}: {:#04x}", a, b);
        let maybe_opcode = Opcode::try_from(b);
        if maybe_opcode.is_ok() {
            println!("  # {:?}", maybe_opcode.unwrap());
        } else {
            println!("");
        }
    }
}

pub fn set_breakpoint(cpu: &mut CPU, address: u16) {
    cpu.breakpoints.push(address);
}

pub fn drop_to_shell(cpu: &mut CPU) -> rustyline::Result<DebuggerRet> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let mut rl = Editor::<()>::with_config(config)?;
    if rl.load_history(".gbdb_history").is_err() {
        println!("no history");
    }

    let mut ret = DebuggerRet::Continue;

    loop {
        let readline = rl.readline("(gbdb) ");

        match readline {
            Ok(line) => {
                let l: &str;
                if line == "" {
                    l = rl.history().last().unwrap();
                } else {
                    l = line.as_str();
                    rl.add_history_entry(line.as_str());
                }

                let l_split: Vec<&str> = l.split(" ").collect();
                if l_split.len() < 1 {
                    continue;
                }

                match l_split[0] {
                    "p"|"print" => print_registers(cpu),
                    "s"|"step"|"next"|"n" => { ret = DebuggerRet::Step; break },
                    "l"|"list" => list_assembly(cpu),
                    "c"|"continue" => { ret = DebuggerRet::Continue; break },
                    "b"|"bp"|"breakpoint" => {
                        if l_split.len() < 2 {
                            println!("Usage: {} ADDRESS", l_split[0]);
                            continue;
                        }

                        let address = l_split[1].parse::<u16>().unwrap();

                        set_breakpoint(cpu, address);
                    },
                    &_ => println!("{}: Command not found", line.as_str()),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting...");
                process::exit(1);
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
            Err(_) => println!("No input"),
        }
    }
    rl.append_history(".gbdb_history")?;

    return Ok(ret);
}
