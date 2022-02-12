use std::env;
use std::fs;
use std::process;
use regex::Regex;
use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref HASHMAP: HashMap<&'static str, (u8, &'static str)> = HashMap::from([
        ("add",  ( 0, "dst"  )),
        ("sub",  ( 1, "dst"  )),
        ("mul",  ( 2, "dst"  )),
        ("div",  ( 3, "dst"  )),
        ("rem",  ( 4, "dst"  )),
        ("and",  ( 5, "dst"  )),
        ("or",   ( 6, "dst"  )),
        ("xor",  ( 7, "dst"  )),
        ("nand", ( 8, "dst"  )),
        ("nor",  ( 9, "dst"  )),
        ("not",  ( 10, "ds"  )),
        ("lsl",  ( 11, "dst" )),
        ("lsr",  ( 12, "dst" )),
        ("asr",  ( 13, "dst" )),
        ("rol",  ( 14, "dst" )),
        ("ror",  ( 15, "dst" )),
        ("addi", ( 16, "dsi" )),
        ("subi", ( 17, "dsi" )),
        ("muli", ( 18, "dsi" )),
        ("divi", ( 19, "dsi" )),
        ("remi", ( 20, "dsi" )),
        ("andi", ( 21, "dsi" )),
        ("ori",  ( 22, "dsi" )),
        ("xori", ( 23, "dsi" )),
        ("nandi", ( 24, "dsi")),
        ("nori", ( 25, "dsi" )),
        ("lsli", ( 26, "dsi" )),
        ("lsri", ( 27, "dsi" )),
        ("asri", ( 28, "dsi" )),
        ("roli", ( 29, "dsi" )),
        ("rori", ( 30, "dsi" )),
        ("addc", ( 31, "dsti")),  
        ("subc", ( 32, "dsti")), 
        ("jeq",  ( 33, "sti" )),
        ("jne",  ( 34, "sti" )),
        ("jgt",  ( 35, "sti" )),
        ("jle",  ( 36, "sti" )),
        ("jlt",  ( 37, "sti" )),
        ("jge",  ( 38, "sti" )),
        ("jeqz", ( 39, "si"  )),
        ("jnez", ( 40, "si"  )),
        ("jgtz", ( 41, "si"  )),
        ("jlez", ( 42, "si"  )),
        ("jltz", ( 43, "si"  )),
        ("jgez", ( 44, "si"  )),
        ("jmp",  ( 45, "i"   )),
        ("beq",  ( 46, "stI" )),
        ("bne",  ( 47, "stI" )),
        ("bgt",  ( 48, "stI" )),
        ("ble",  ( 49, "stI" )),
        ("blt",  ( 50, "stI" )),
        ("bge",  ( 51, "stI" )),
        ("beqz", ( 52, "sI"  )),
        ("bnez", ( 53, "sI"  )),
        ("bgtz", ( 54, "sI"  )),
        ("blez", ( 55, "sI"  )),
        ("bltz", ( 56, "sI"  )),
        ("bgez", ( 57, "sI"  )),
        ("br",   ( 58, "I"   )),
        ("jsr",  ( 59, "i"   )),
        ("rts",  ( 60, ""    )),
        ("inc",  ( 61, "s"   )),
        ("dec",  ( 62, "s"   )),
        ("li",   ( 63, "di"  )),
        ("lw",   ( 64, "di"  )),
        ("sw",   ( 65, "di"  )),
        ("lwi",  ( 66, "dsi" )),
        ("swi",  ( 67, "dsi" )),
        ("push", ( 68, "d",	 )),
        ("pop",  ( 69, "d",	 )),
        ("move", ( 70, "ds"  )),
        ("clr",  ( 71, "s"   )),
        ("neg",  ( 72, "s"   )),
        ("lwri", ( 73, "dst" )),    
        ("swri", ( 74, "dst" )),	 
    ]);
}

fn parse_register(text: &str) -> u16 {
    let register_regex = Regex::new(r"^r([0-7])$").unwrap();
    let m = Regex::captures(&register_regex, text).expect("Wrong argument");
    m[1].parse::<u16>().unwrap()
}

fn parse_immed(text: &str) -> i16 {
    let number_regex = Regex::new(r"^(?:0([xb]))?([+-]?\d+)$").unwrap();
    let m = Regex::captures(&number_regex, text).expect("Not a number");
    let num = m.get(2).expect("Not a number").as_str();
    match m.get(1) {
        Some(radix) => {
            let radix: u32 = match radix.as_str() {
                "x" => 16,
                "b" => 2,
                _ => unreachable!("Oh no, that's not possible!")
            };
            i16::from_str_radix(&num, radix).expect("Number can't fit into 16 bits!")
        },
        None => {
            num.parse::<i16>().expect("Number can't fit into 16 bits!")
        }
    }
}

fn main() {
    let re = Regex::new(r"^(?:(?P<oznaka>\w+):)?\s*(?P<ukaz>[a-zA-Z]+)(?:\s+([+-]?\w+))?(?:\s*,\s*([+-]?\w+))?(?:\s*,\s*([+-]?\w+))?(?:\s*,\s*([+-]?\w+))?\s*#*.*$").unwrap();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Pot do datoteke ni podana!");
        process::exit(1);
    }
    let file_path = &args[1];
    let contents = fs::read_to_string(file_path)
        .expect("Something went wrong reading the file");

    let mut address: u16 = 0;
    let mut labels = HashMap::<String, u16>::new();
    let mut instr = Vec::<u16>::new();

    for line in contents.lines() {
        
        for cap in re.captures_iter(line) {
            if let Some(oznaka) = cap.name("oznaka") {
                labels.insert(String::from(oznaka.as_str()), address);
            }
            if let Some(ukaz) = cap.name("ukaz") {
                let ukaz = ukaz.as_str().to_lowercase();
                let (_, format) = HASHMAP.get(&ukaz.as_ref()).expect(format!("Unknown instruction: {ukaz}").as_str());
                let format = format.to_string();
                if format.contains('i') || format.contains('I') {
                    address += 1;
                }
                address += 1;
                
            }
        }
    }

    address = 0;
    
    for line in contents.lines() {
        
        for cap in re.captures_iter(line) {
            if let Some(ukaz) = cap.name("ukaz") {
                let mut instr: u16 = 0;
                let mut immed: Option<i16> = None;
                let ukaz = ukaz.as_str().to_lowercase();
                let (opcode, format) = HASHMAP.get(&ukaz.as_ref()).expect(format!("Unknown instruction: {ukaz}").as_str());
                let opcode = opcode.clone();
                let format = format.clone();
                instr |= (opcode as u16) << 9;
                
                let mut arguments: Vec<String> = Vec::new();
                for i in 3..=6 {
                    if let Some(arg) = cap.get(i) {
                        let arg = arg.as_str().to_lowercase();
                        arguments.push(arg);
                    }
                }

                if arguments.len() > format.len() {
                    panic!("Too many arguments!")
                } else if arguments.len() < format.len() {
                    panic!("Arguments missing!")
                }

                for (i, arg_type) in format.chars().enumerate() {
                    let arg = arguments.get(i).expect("Missing argument");
                    match arg_type {
                        'd' => {
                            let reg = parse_register(&arg);
                            instr |= reg;
                        },
                        's' => {
                            let reg = parse_register(&arg);
                            instr |= reg << 3;
                        },
                        't' => {
                            let reg = parse_register(&arg);
                            instr |= reg << 6;
                        },
                        'i' => {
                            if let Some(addr) = labels.get(arg) {
                                immed = Some(addr.clone() as i16);
                            } else {
                                immed = Some(parse_immed(&arg));
                            }
                        },
                        'I' => {
                            if let Some(addr) = labels.get(arg) {
                                immed = Some(addr.clone() as i16 - address as i16 - 1);
                            } else  {
                                immed = Some(parse_immed(&arg) - address as i16 - 1);
                            }
                        },
                        _ => unreachable!("This shouldn't happen!")
                    }
                }
                print!("{:#06x}: ", address);
                print!("{instr:#06x} {instr:#018b}");
                println!("   {line}");
                address += 1;

                if let Some(immed) = immed {
                    print!("{:#06x}: ", address);
                    println!("{immed:#06x} {immed:#018b}");
                    address += 1;
                }
            }
        }
        
    }
}
