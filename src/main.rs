use std::env;
use std::process::exit;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::prelude::*;
use std::collections::HashMap;

enum OpcodeType {
    LocRequired,
    LocAndLabelRequred,
    NothingRequired,
    NotAnOpcode
}

static PROG_NAME: &'static str = "acslasmc v0.1";
static PROG_NAME_LINE: &'static str = "acslasmc v0.1 -- ACSL Assembly to C compiler";

static HEADER: &'static str = "\
#include<stdio.h>

int get_mem_size();
int MOD = 1000000;

int main() {
    int MEM_SIZE = get_mem_size();
    int acc = 0;
    int mem[MEM_SIZE];
    for (int i=0; i<MEM_SIZE; i++) {
        mem[i] = 0;
    }
";

static PRE_FOOTER: &'static str = "\
}

int get_mem_size() {
";

static FOOTER: &'static str = "}";

fn main() {
    let status = encapsulated_main();
    exit(status);
}

fn encapsulated_main() -> i32 {
    println!("{}", PROG_NAME_LINE);
    let mut args = env::args();

    let input_file_path = match args.nth(1) {
        Some(x) => x,
        None => {
            println!("Please pass a source file with ACSL assembly code and a target file for generated C code as arguments.");
            return 2;
        }
    }; 

    let output_file_path = match args.next() {
        Some(x) => x,
        None => format!("{}_output.c", input_file_path)
    };

    let mut reader = BufReader::new(
        match File::open(&input_file_path) {
            Ok(x) => x,
            Err(_) => {
                println!("Failed to open source file '{}'.", input_file_path);
                return 3;
            }
        }
    );
    println!("Source file: '{}'.", input_file_path);

    let mut writer = BufWriter::new(
        match File::create(&output_file_path) {
            Ok(x) => x,
            Err(_) => {
                println!("Failed to open output file '{}'.", output_file_path);
                return 3;
            }
        }
    );
    println!("Output file: '{}'.", output_file_path);

    let mut counter: u32 = 0;
    let mut var_map: HashMap<String, u32> = HashMap::new();

    let mut buffer = String::new();
    let mut line_count: u32  = 1;

    // Write the beginnings of the C code
    writeln!(writer, "// generated from ACSL assembly source '{}' by {}\n{}", input_file_path, PROG_NAME, HEADER).unwrap();

    loop {
        match reader.read_line(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read <= 0 {
                    break;
                }
            },
            Err(_) => panic!("Failed to read line!")
        }

        {
            let comps: Vec<_> = buffer.trim().split(' ').collect();
            let translated = match process_comps(&comps, &mut var_map, &mut counter) {
                Ok(x) => x,
                Err(x) => {
                    println!("line {}: error: {}", line_count, x);
                    println!("Compilation failed.");
                    return 1;
                }
            };
            writeln!(writer, "    {}", translated).unwrap();
        }
        line_count += 1;
        buffer.clear();
    }

    // Write some ending C code
    writeln!(writer, "{}    return {};\n{}\n", PRE_FOOTER, var_map.len(), FOOTER).unwrap();
    println!("You may pass the generated C source to a C compiler to run it: e.g.");
    println!("gcc {of} -o {of}.out && ./{of}.out", of = output_file_path);
    println!("Compilation complete.");
    return 0;
}

fn process_comps(comps: &Vec<&str>, var_map: &mut HashMap<String, u32>, counter: &mut u32) -> Result<String, String> {
    match opcode_type(match comps.get(0) {
        Some(x) => x,
        None => return Ok(String::new()) // blank line, go to next one
    }) {
        OpcodeType::LocRequired => return trans(var_map, counter, "", comps[0], match comps.get(1) {
            None => return Err(format!("missing loc: loc is required for opcode {}", comps[0])),
            Some(x) => x
        }),
        OpcodeType::NothingRequired => return trans(var_map, counter, "", comps[0], ""),
        OpcodeType::LocAndLabelRequred => return Err(format!("missing label: label is required for opcode {}", comps[0])),
        OpcodeType::NotAnOpcode => match opcode_type(match comps.get(1) {
            Some(x) => x,
            None => return Err(format!("missing opcode, only label provided"))
        }) {
            OpcodeType::LocRequired | OpcodeType::LocAndLabelRequred => return trans(var_map, counter, comps[0], comps[1], match comps.get(2) {
                Some(x) => x,
                None => return Err(format!("missing loc: loc is required for opcode {}", comps[1]))
            }),
            OpcodeType::NothingRequired => return trans(var_map, counter, comps[0], comps[1], ""),
            OpcodeType::NotAnOpcode => return Err(format!("invalid opcode: {}", comps[1]))
        }
    }
}

fn trans(var_map: &mut HashMap<String, u32>, counter: &mut u32, label: &str, opcode: &str, loc: &str) -> Result<String, String> {
    let read_loc: String = match loc.chars().nth(0) {
        Some(x) if x == '=' => loc.split_at(1).1.to_owned(),
        Some(_) => match var_map.get(&loc.to_owned()) {
            Some(y) => format!("mem[{}]", y),
            None => String::new()
        },
        None => String::new()
    };

    let action = match opcode {
        "LOAD" => format!("acc = {read_loc};", read_loc = read_loc),
        "STORE" => {
            let res_loc = req_mem(var_map, counter, loc);
            format!("mem[{res_loc}] = acc;", res_loc = res_loc)
        },
        "ADD" => format!("acc = (acc + {read_loc}) % MOD;", read_loc = read_loc),
        "SUB" => format!("acc = (acc - {read_loc}) % MOD;", read_loc = read_loc),
        "MULT" => format!("acc = (acc * {read_loc}) % MOD;", read_loc = read_loc),
        "DIV" => format!("acc = (acc / {read_loc}) % MOD;", read_loc = read_loc),
        "BE" => format!("if (acc == 0) goto {raw_loc};", raw_loc = loc),
        "BU" => format!("if (acc > 0) goto {raw_loc};", raw_loc = loc),
        "BL" => format!("if (acc < 0) goto {raw_loc};", raw_loc = loc),
        "END" => "return 0;".to_owned(), // Ignore LOC field
        // TODO: Implement READ opcode
        "READ" => return Err("READ is unimplemented".to_owned()),
        "PRINT" => format!("printf(\"%d\\n\", {read_loc});", read_loc = read_loc),
        "DC" => {
            let label_loc = req_mem(var_map, counter, label);
            format!("mem[{label_loc}] = {raw_loc};", label_loc = label_loc, raw_loc = loc)
        },
        _ => "".to_owned()
    };
    // Process the labels
    let mut statement = String::new();
    if opcode != "DC" && label != "" {
        statement.push_str(label);
        statement.push_str(":;\n    ");
    }
    Ok(format!("{}{}", statement, action))
}

fn req_mem(var_map: &mut HashMap<String, u32>, counter: &mut u32, loc: &str) -> u32 {
    let owned_loc = loc.to_owned();
    match var_map.get(&owned_loc) {
        Some(x) => return *x,
        None => ()
    }
    var_map.insert(owned_loc, *counter);
    *counter += 1;
    *counter - 1
}

fn opcode_type(potential_opcode: &str) -> OpcodeType {
    match potential_opcode {
        "LOAD" | "STORE" | "ADD" | "SUB" | "MULT" | "DIV" | "BE" | "BG" | "BL" | "BU" | "READ" | "PRINT" => OpcodeType::LocRequired,
        "END" => OpcodeType::NothingRequired,
        "DC" => OpcodeType::LocAndLabelRequred,
         _ => OpcodeType::NotAnOpcode
    }
}

