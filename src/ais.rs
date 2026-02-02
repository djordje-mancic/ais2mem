use std::{cmp::min, fs::File, io::{BufReader, BufWriter, Read, Seek, Write}, process::exit};

use crate::options::ProgramOptions;

const AIS_MAGIC_WORD: u32 = 0x41504954;
const CMD_SECTION_LOAD: u32 = 0x58535901;
const CMD_CRC_ENABLE: u32 = 0x58535903;
const CMD_CRC_DISABLE: u32 = 0x58535904;
const CMD_JUMP_CLOSE: u32 = 0x58535906;

fn read_word(file: &mut BufReader<File>) -> u32 {
    let mut buf = [0u8; 4];
    if let Err(e) = file.read_exact(&mut buf) {
        eprintln!("File specified is malformed: {e}");
        exit(-1);
    }
    u32::from_le_bytes(buf)
}

pub fn is_ais(reader: &mut BufReader<File>) -> bool {
    return read_word( reader) != AIS_MAGIC_WORD;
}

pub fn convert(
    options: &ProgramOptions,
    input_reader: &mut BufReader<File>, 
    output_writer: &mut BufWriter<File>,
) {
     // Command read loop
    loop {
        let command = read_word(input_reader);
        if options.verbose {
            println!("Command {command:x}");
        }
        match command {
            CMD_SECTION_LOAD => {
                let address = read_word(input_reader);
                let size = read_word(input_reader);
                if options.verbose {
                    println!("Section Load");
                    println!("Address: {address:x}");
                    println!("Size: {size} bytes");
                }
                let mut bytes_left = size as usize;

                if let Err(e) = output_writer.seek(std::io::SeekFrom::Start(address as u64)) {
                    eprintln!("Error seeking to address in output file: {e}");
                    exit(-1);
                }

                const READ_BUFFER_SIZE: usize = 4096;
                let mut buffer = [0u8; READ_BUFFER_SIZE];
                while bytes_left > 0 {
                    let read_size = min(bytes_left, READ_BUFFER_SIZE);
                    if let Err(e) = input_reader.read_exact(&mut buffer[0..read_size]) {
                        eprintln!("Error reading file contents: {e}");
                        exit(-1);
                    }
                    if let Err(e) = output_writer.write_all(&buffer[0..read_size]) {
                        eprintln!("Error writing to output file: {e}");
                        exit(-1);
                    }
                    bytes_left -= read_size;
                }
            }
            CMD_CRC_ENABLE => {
                // Does nothing, TODO
                if options.verbose {
                    println!("Enable CRC");
                }
            }
            CMD_CRC_DISABLE => {
                // Does nothing, TODO
                if options.verbose {
                    println!("Disable CRC");
                }
            }
            CMD_JUMP_CLOSE => {
                let address = read_word(input_reader);
                if options.verbose {
                    println!("Jump & Close");
                    println!("Address: {address:x}");
                }
                println!("Conversion done!");
                println!("Entry point: {address:x}");
                break;
            }
            _ => {
                eprintln!("File contains unsupported command {command:x}");
                exit(-1);
            }
        }
    }
}