use std::env;
use std::process;
use std::fs;
use std::collections::HashMap;
use regex::Regex;
use std::path::Path;
use std::fs::File;
use std::io::Write;

#[macro_use]
extern crate lazy_static;

// Address is 8bit
// Data is 32 bit but actually only 23 bits are used

lazy_static! {
    static ref MICROOP_REGEX: Regex = Regex::new(r"^(?:(\w+):)?\s*([\w\s=]+)?(?:\s*,\s*([\w\s]+))?\s*#*.*$").unwrap();
    static ref CONTROL_SIGNAL_REGEX: Regex = Regex::new(r"(\w+)=(\w+)").unwrap();
    static ref JUMP_IF_REGEX: Regex = Regex::new(r"^if\s+(n|z|c|corz)\s+then\s+(\w+)(?:\s+else\s+(\w+))?$").unwrap();
    static ref JUMP_GOTO_REGEX: Regex = Regex::new(r"^goto\s+(\w+)$").unwrap();
}

fn parse_control_signal(control_signal: &str, value: &str) -> Option<u32> {
    match control_signal {
        "aluop" => match value {
            "add"  =>  Some(0),
            "sub"  =>  Some(1),
            "mul"  =>  Some(2),
            "div"  =>  Some(3),
            "rem"  =>  Some(4),
            "and"  =>  Some(5),
            "or"   =>  Some(6),
            "xor"  =>  Some(7),
            "nand" =>  Some(8),
            "nor"  =>  Some(9),
            "not"  => Some(10),
            "lsl"  => Some(11),
            "lsr"  => Some(12),
            "asr"  => Some(13),
            "rol"  => Some(14),
            "ror"  => Some(15),
            _      => None,
        },
        "op2sel" => match value {
            "treg"   =>  Some(0 << 4),
            "immed"  =>  Some(1 << 4),
            "const0" =>  Some(2 << 4),
            "const1" =>  Some(3 << 4),
            _        =>  None,
        },
        "datawrite" => match value {
            "0" =>  Some(0 << 6),
            "1" =>  Some(1 << 6),
            _   =>  None,
        },
        "addrsel" => match value {
            "pc"     =>  Some(0 << 7),
            "immed"  =>  Some(1 << 7),
            "aluout" =>  Some(2 << 7),
            "sreg"   =>  Some(3 << 7),
            _        =>  None,
        },
        "pcsel" => match value {
            "pc"      =>  Some(0 << 9),
            "immed"   =>  Some(1 << 9),
            "pcimmed" =>  Some(2 << 9),
            "sreg"    =>  Some(3 << 9),
            _         =>  None,
        },
        "pcload" => match value {
            "0" =>  Some(0 << 11),
            "1" =>  Some(1 << 11),
            _          =>  None,
        },
        "dwrite" => match value {
            "0" =>  Some(0 << 12),
            "1" =>  Some(1 << 12),
            _   =>  None,
        },
        "irload" => match value {
            "0" =>  Some(0 << 13),
            "1" =>  Some(1 << 13),
            _   =>  None,
        },
        "imload" => match value {
            "0" =>  Some(0 << 14),
            "1" =>  Some(1 << 14),
            _   =>  None,
        },
        "regsrc" => match value {
            "databus" =>  Some(0 << 15),
            "immed"   =>  Some(1 << 15),
            "aluout"  =>  Some(2 << 15),
            "sreg"    =>  Some(3 << 15),
            _         =>  None,
        },
        "cond" => match value {
            "c"    =>  Some(0 << 17),
            "corz" =>  Some(1 << 17),
            "z"    =>  Some(2 << 17),
            "n"    =>  Some(3 << 17),
            _      =>  None,
        },
        "indexsel" => match value {
            "0"      =>  Some(0 << 19),
            "opcode" =>  Some(1 << 19),
            _        =>  None,
        },
        "datasel" => match value {
            "pc"     =>  Some(0 << 20),
            "dreg"   =>  Some(1 << 20),
            "treg"   =>  Some(2 << 20),
            "aluout" =>  Some(3 << 20),
            _        =>  None,
        },
        "swrite" => match value {
            "0" =>  Some(0 << 22),
            "1" =>  Some(1 << 22),
            _   =>  None,
        },
        _ => None,
    }
}

fn write_to_file<P, S>(path: P, bytes: Vec<S>)
where
    P: AsRef<Path>,
    S: Into<u32>
{
    let mut file = File::create(path).expect("No permission to create file!");
    file.write_all(b"v2.0 raw\n").unwrap();
    for byte in bytes {
        file.write_all(format!("{:x}\n", byte.into()).as_bytes()).unwrap();
    }
    file.flush().expect("Something wrong happened writing to the file");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("File path not specified!");
        process::exit(1);
    }
    let file_path = &args[1];
    let contents = fs::read_to_string(file_path)
        .expect("Something went wrong reading the file");

    let mut labels = HashMap::<&str, u8>::new();
    let mut instructions: Vec<u32> = vec![0; 256];
    let mut jumps: Vec<u16> = vec![0; 256];
    let mut original_lines: Vec<String> = vec![String::new(); 256];
    let mut offset = 0;
    let mut next_addr = 0;

    // Process labels
    for line in contents.lines() {
        let cap = Regex::captures(&MICROOP_REGEX, line).expect("Not a valid microop");
        let (label, control_signals, jump) = (cap.get(1), cap.get(2), cap.get(3));
        let mut addr: u8 = next_addr.clone();

        if control_signals.is_none() && jump.is_none() {
            continue;
        }

        if let Some(label) = label {
            let label = label.as_str();
            if labels.contains_key(label) {
                panic!("Label {label} is already defined!");
            }
            if let Ok(label) = label.parse::<u8>() {
                addr = offset + label;
            } else {
                labels.insert(label, addr);
                next_addr += 1;
            }
        } else {
            next_addr += 1;
        }

        if let Some(jump) = jump {
            let jump = jump.as_str().trim();
            if jump == "opcode_jump" {
                offset = next_addr.clone();
                next_addr = offset + 128;
            }
        }

    }

    offset = 0;
    next_addr = 0;
    
    // Generate code
    for line in contents.lines() {
        let cap = Regex::captures(&MICROOP_REGEX, line).expect("Not a valid microop");
        let (label, control_signals, jump) = (cap.get(1), cap.get(2), cap.get(3));
        let mut original_line = String::new();
        let mut instr: u32 = 0;
        let mut jmp: u16 = 0;
        let mut addr: u8 = next_addr.clone();

        if control_signals.is_none() && jump.is_none() {
            continue;
        }

        if let Some(label) = label {
            let label = label.as_str();
            original_line.push_str(format!("{}: ",label).as_str());
            if let Ok(label) = label.parse::<u8>() {
                addr = offset + label;
            } else {
                next_addr += 1;
            }
        } else {
            next_addr += 1;
        }

        if let Some(control_signals) = control_signals {
            original_line.push_str(control_signals.as_str());
            for capture in CONTROL_SIGNAL_REGEX.captures_iter(control_signals.as_str()) {
                let control_bits = parse_control_signal(capture.get(1).unwrap().as_str(), capture.get(2).unwrap().as_str()).expect("Wrong control signal");
                instr |= control_bits;
            }
            
        }

        if let Some(jump) = jump {
            let jump = jump.as_str().trim();
            original_line.push_str(format!(", {}", jump).as_str());
            if jump == "opcode_jump" {
                let control_bits = parse_control_signal("indexsel", "opcode").unwrap();
                instr |= control_bits;
                jmp = ((next_addr as u16) << 8) | (next_addr as u16);
                offset = next_addr.clone();
                next_addr = offset + 128;
            } else if let Some(jump) = JUMP_GOTO_REGEX.captures(jump) {
                if let Some(label) = jump.get(1) {
                    let label_addr = labels.get(label.as_str()).expect("Label is not defined!").clone();
                    jmp = ((label_addr as u16) << 8) | (label_addr as u16);
                }
            } else if let Some(jump) = JUMP_IF_REGEX.captures(jump) {
                let control_bits = parse_control_signal("cond", jump.get(1).unwrap().as_str()).unwrap();
                instr |= control_bits;
                let if_jump = jump.get(2).unwrap().as_str();
                let if_addr = labels.get(if_jump).expect("Label is not defined!").clone();
                let mut else_addr: u8 = 0;
                if let Some(else_jump) = jump.get(3) {
                    let else_jump = else_jump.as_str();
                    else_addr = labels.get(else_jump).expect("Label is not defined!").clone();
                } else {
                    else_addr = next_addr.clone();
                }
                jmp = ((if_addr as u16) << 8) | (else_addr as u16);
            } else {
                panic!("Can't decode jump instruction")
            }
        } else {
            // jump to next
            jmp = ((next_addr as u16) << 8) | (next_addr as u16);
        }
        
        instructions[addr as usize] = instr;
        original_lines[addr as usize] = original_line;
        jumps[addr as usize] = jmp;
    }

    // Print out the code
    for addr in 0..256 {
        if instructions[addr] == 0 { continue };
        println!("{:02x}: {:08x} {:04x}       # {}", addr, instructions[addr], jumps[addr], original_lines[addr]);
    }

    // Write to ROM files
    let path = Path::new(file_path).with_file_name("ucontrol.rom");
    write_to_file(path, instructions);

    let path = Path::new(file_path).with_file_name("udecision.rom");
    write_to_file(path, jumps);
}
