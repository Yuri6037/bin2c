// Copyright 2020 Yuri6037

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

#![allow(irrefutable_let_patterns)]

use std::fs::File;
use std::io::prelude::*;
use std::process;
use std::path::Path;
use std::io;
use clap::clap_app;

const MAX_BYTES_PER_BUFFER: usize = 2097152;

fn write_header(const_name: &str, output: &str, count: u32, total_len: usize, write_blocks_heading: bool) -> io::Result<()> {
    let mut outfile = File::create(&Path::new(output))?;
    let head = "#include <stdlib.h>\n\n#ifdef __cplusplus\nextern \"C\"\n{\n#endif\n\n";
    let foot = "\n#ifdef __cplusplus\n}\n#endif\n";
    let foot2 = format!("#define {}_SIZE {}\n", const_name, total_len);
    let foot3 = format!("#define {}_BLOCK_COUNT {}\n", const_name, count - 1);

    outfile.write(head.as_bytes())?;
    for i in 1..count {
        let s = format!("extern const size_t {}_{}_SIZE;\nextern const unsigned char {}_{}[];\n", const_name, i, const_name, i);
        outfile.write(s.as_bytes())?;
    }
    if write_blocks_heading {
        outfile.write(format!("\nstatic const unsigned char *{}_BLOCKS[] = {{\n", const_name).as_bytes())?;
        for i in 1..count {
            if i == count - 1 {
                let s = format!("\t{}_{}\n", const_name, i);
                outfile.write(s.as_bytes())?;
            } else {
                let s = format!("\t{}_{},\n", const_name, i);
                outfile.write(s.as_bytes())?;
            }
        }
        outfile.write("};\n".as_bytes())?;
        outfile.write(format!("\nstatic const size_t *{}_BLOCK_SIZES[] = {{\n", const_name).as_bytes())?;
        for i in 1..count {
            if i == count - 1 {
                let s = format!("\t&{}_{}_SIZE\n", const_name, i);
                outfile.write(s.as_bytes())?;
            } else {
                let s = format!("\t&{}_{}_SIZE,\n", const_name, i);
                outfile.write(s.as_bytes())?;
            }
        }
        outfile.write("};\n".as_bytes())?;
    }
    outfile.write(foot.as_bytes())?;
    outfile.write(foot2.as_bytes())?;
    if write_blocks_heading {
        outfile.write(foot3.as_bytes())?;
    }
    return Ok(());
}

fn write_file_buffer(file: &mut File, hfile: &str, const_name: &str, id: u32) -> io::Result<(usize, String)> {
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

fn bin_to_c(const_name: &str, hfile: &str, input: &str, output: &str) -> io::Result<(usize, u32)> {
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
    let matches = clap_app!(bin2c =>
        (version: "1.0")
        (author: "Yuri6037 <https://github.com/Yuri6037/bin2c>")
        (about: "A cross platform Rust version of the UNIX tool bin2c")
        (@arg INPUT: +required "The input file to convert to a C file")
        (@arg OUTPUT: +required "The output base file name")
        (@arg const_name: -c --constname +takes_value "The base name for the generated constants")
        (@arg write_blocks_heading: -b --blocks "Write blocks heading")
    ).get_matches();
    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();
    let const_name = match matches.value_of("const_name") {
        Some(v) => v,
        None => "BINARY_DATA"
    };
    let hfile = String::from(output) + ".h";

    match bin_to_c(&const_name, &hfile, input, output) {
        Err(e) => {
            println!("An error has occured: {}", e);
            process::exit(1);
        },
        Ok((total_len, count)) => {
            println!("Wrote C source(s)");
            match write_header(&const_name, &hfile, count, total_len, matches.is_present("write_blocks_heading")) {
                Err(e) => {
                    println!("An error has occured: {}", e);
                    process::exit(1);
                },
                Ok(()) => println!("Wrote C header")
            }
        }
    }
}
