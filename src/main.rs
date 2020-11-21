#![allow(irrefutable_let_patterns)]

use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;
use std::env;
use std::process;
use std::path::Path;
use std::io;

const MAX_BYTES_PER_BUFFER: usize = 2097152;

fn write_header(const_name: &str, output: &str, count: u32, total_len: usize) -> io::Result<()>
{
    let mut outfile = File::create(&Path::new(output))?;
    let head = "#include <stdlib.h>\n\n#ifdef __cplusplus\nextern \"C\"\n{\n#endif\n";
    let foot = "\n#ifdef __cplusplus\n}\n#endif\n";
    let foot2 = format!("#define {}_SIZE {}", const_name, total_len);

    outfile.write(head.as_bytes())?;
    for i in 1..count {
        let s = format!("extern const size_t {}_{}_SIZE;\nextern const unsigned char {}_{}[];\n", const_name, i, const_name, i);
        outfile.write(s.as_bytes())?;
    }
    outfile.write(foot.as_bytes())?;
    outfile.write(foot2.as_bytes())?;
    return Ok(());
}

fn write_file_buffer(file: &mut File, hfile: &str, const_name: &str, id: u32) -> io::Result<(usize, String)>
{
    let mut buffer: [u8;512] = [0; 512];
    let mut globalstr = String::from("#include \"") + hfile + "\"\n\nconst unsigned char " + &format!("{}_{}", const_name, id) + "[] = {";
    let mut globallen: usize = 0;

    while let size = file.read(&mut buffer)? {
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
        if globallen >= MAX_BYTES_PER_BUFFER {
            break;
        }
    }
    globalstr = String::from(&globalstr[0..globalstr.len() - 2]);
    globalstr += "};";
    globalstr += &format!("\nconst size_t {}_{}_SIZE = {};", const_name, id, globallen);
    globalstr += "\n";
    return Ok((globallen, globalstr));
}

fn bin_to_c(const_name: &str, hfile: &str, input: &str, output: &str) -> io::Result<(usize, u32)>
{
    let mut len: usize = 0;
    let mut count: u32 = 1;
    let mut infile = File::open(&Path::new(input))?;

    while let (globallen, globalstr) = write_file_buffer(&mut infile, hfile, const_name, count)? {
        if globallen == 0 {
            break;
        }
        let path: String = [output, ".", &count.to_string(), ".c"].join("");
        let mut outfile = File::create(path)?;
        outfile.write(globalstr.as_bytes())?;
        count += 1;
        len += globallen;
        if globallen < MAX_BYTES_PER_BUFFER {
            break;
        }
    }
    return Ok((len, count));
}

fn main() {
    let const_name = "BINARY_DATA";
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage <input file> <output file>");
        process::exit(1);
    }
    let hfile = String::from(&args[2]) + ".h";
    match bin_to_c(&const_name, &hfile, &args[1], &args[2]) {
        Err(e) => {
            println!("An error has occured: {}", e);
            process::exit(1);
        },
        Ok((total_len, count)) => {
            println!("Wrote C source");
            match write_header(&const_name, &hfile, count, total_len) {
                Err(e) => {
                    println!("An error has occured: {}", e);
                    process::exit(1);
                },
                Ok(()) => println!("Wrote C header")
            }
        }
    }
}
