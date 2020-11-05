#![allow(irrefutable_let_patterns)]

use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;
use std::env;
use std::process;
use std::path::Path;
use std::io;

fn write_header(const_name: &str, output: &str) -> io::Result<()>
{
    let mut outfile = File::create(&Path::new(output))?;
    let s = format!("#include <stdlib.h>\n\nextern const size_t {}_SIZE;\nextern const char {}[];", const_name, const_name);

    outfile.write(s.as_bytes())?;
    return Ok(());
}

fn bin_to_c(const_name: &str, input: &str, output: &str) -> io::Result<()>
{
    let mut buffer: [u8;512] = [0; 512];
    let mut infile = File::open(&Path::new(input))?;
    let mut outfile = File::create(&Path::new(output))?;
    let mut globalstr = String::from("#include <stdlib.h>\n\nconst char ") + const_name + "[] = {";
    let mut globallen: usize = 0;

    while let size = infile.read(&mut buffer)? {
        if size <= 0 {
            break;
        }
        for i in 0..size {
            let s = hex::encode([buffer[i]]);
            globalstr += "0x";
            globalstr += &s;
            globalstr += ", ";
        }
        globallen += size;
    }
    globalstr = String::from(&globalstr[0..globalstr.len() - 2]);
    globalstr += "};";
    globalstr += &format!("\nconst size_t {}_SIZE = {};", const_name, globallen);
    globalstr += "\n";
    outfile.write(globalstr.as_bytes())?;
    return Ok(());
}

fn main() {
    let const_name = "BINARY_DATA";
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage <input file> <output file>");
        process::exit(1);
    }
    let cfile = String::from(&args[2]) + ".c";
    let hfile = String::from(&args[2]) + ".h";
    match bin_to_c(&const_name, &args[1], &cfile) {
        Err(e) => {
            println!("An error has occured: {}", e);
            process::exit(1);
        },
        Ok(()) => println!("Wrote C source")
    }
    match write_header(&const_name, &hfile) {
        Err(e) => {
            println!("An error has occured: {}", e);
            process::exit(1);
        },
        Ok(()) => println!("Wrote C header")
    }
}
