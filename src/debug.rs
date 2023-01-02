use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, EditMode, Editor};
use std::process;
use parse_int::parse;

use crate::cpu::CPU;
use crate::cpu::Opcode;
use crate::registers::Flag;

#[derive(Debug)]
pub enum DebuggerRet {
    Step,
    Continue,
}

pub enum Registers {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    SP,
    PC,
}

impl TryFrom<&str> for Registers {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "A"|"a" => Ok(Registers::A),
            "B"|"b" => Ok(Registers::B),
            "C"|"c" => Ok(Registers::C),
            "D"|"d" => Ok(Registers::D),
            "E"|"e" => Ok(Registers::E),
            "F"|"f" => Ok(Registers::F),
            "H"|"h" => Ok(Registers::H),
            "L"|"l" => Ok(Registers::L),
            "SP"|"sp" => Ok(Registers::SP),
            "PC"|"pc" => Ok(Registers::PC),
            _ => Err("invalid register"),
        }
    }
}

pub fn print_registers(cpu: &CPU) {
    println!("A: {:#04x}, B: {:#04x}, C: {:#04x}, D: {:#04x}, E: {:#04x}, H: {:#04x}, L: {:#04x}, PC: {:#04x}, SP: {:#04x}",
             cpu.reg.a, cpu.reg.b, cpu.reg.c, cpu.reg.d, cpu.reg.e, cpu.reg.h, cpu.reg.l, cpu.reg.pc, cpu.reg.sp);
    println!("");
    print!("Z: {}, ", cpu.reg.get_flag(Flag::Z));
    print!("N: {}, ", cpu.reg.get_flag(Flag::N));
    print!("H: {}, ", cpu.reg.get_flag(Flag::H));
    println!("C: {}", cpu.reg.get_flag(Flag::C));
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

    let b = cpu.memory_bus.read_byte(cpu.reg.pc.into());
    print!("{:#04x}:\t{:#04x}", cpu.reg.pc, b);
    let maybe_opcode = Opcode::try_from(b);
    if maybe_opcode.is_ok() {
        println!("  # {:?}", maybe_opcode.unwrap());
    } else {
        println!("");
    }

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
                    "p"|"print" => {
                        if l_split.len() < 2 {
                            print_registers(cpu);
                            continue;
                        }

                        let address = parse::<u16>(l_split[1]);
                        match address {
                            Ok(a) => println!("{:#04x}", cpu.memory_bus.read_byte(a)),
                            Err(_) => { println!("bad number"); continue; },
                        };
                    }
                    "s"|"step"|"next"|"n" => { ret = DebuggerRet::Step; break },
                    "l"|"list" => list_assembly(cpu),
                    "c"|"continue" => { ret = DebuggerRet::Continue; break },
                    "b"|"bp"|"breakpoint" => {
                        if l_split.len() < 2 {
                            println!("Usage: {} ADDRESS", l_split[0]);
                            continue;
                        }

                        let address = parse::<u16>(l_split[1]);
                        match address {
                            Ok(a) => set_breakpoint(cpu, a),
                            Err(_) => { println!("bad number"); continue; },
                        };
                    },
                    "d"|"display" => {
                        if l_split.len() < 2 {
                            println!("Usage: {} REGISTER", l_split[0]);
                            continue;
                        }
                    }
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
