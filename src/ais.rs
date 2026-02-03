use std::{
    cmp::min,
    fs::File,
    io::{BufReader, BufWriter, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write},
    process::exit,
};

use crate::options::ProgramOptions;

// AIS command opcodes
const AIS_MAGIC_WORD: u32 = 0x41504954;
const CMD_SECTION_LOAD: u32 = 0x58535901;
const CMD_SECTION_FILL: u32 = 0x5853590A;
const CMD_VALIDATE_CRC: u32 = 0x58535902;
const CMD_CRC_ENABLE: u32 = 0x58535903;
const CMD_CRC_DISABLE: u32 = 0x58535904;
const CMD_JUMP: u32 = 0x58535905;
const CMD_JUMP_CLOSE: u32 = 0x58535906;

fn read_word(file: &mut BufReader<&File>) -> u32 {
    let mut buf = [0u8; 4];
    if let Err(e) = file.read_exact(&mut buf) {
        eprintln!("File specified is malformed: {e}");
        exit(-1);
    }
    u32::from_le_bytes(buf)
}

pub fn is_ais(reader: &mut BufReader<&File>) -> bool {
    return read_word(reader) == AIS_MAGIC_WORD;
}

pub fn convert(
    options: &ProgramOptions,
    input_reader: &mut BufReader<&File>,
    output_file: &File,
) -> Result<(u32, u32)> {
    let mut entry_point = 0u32;
    let mut lowest_address = u32::MAX;
    let mut sections: Vec<(u32, Vec<u8>)> = vec![];

    // Notifiers for unimplemented features
    let mut crc_notified = false;
    let mut jump_notified = false;

    loop {
        let command = read_word(input_reader);
        if options.verbose {
            println!("Command 0x{command:X}");
        }
        match command {
            CMD_SECTION_LOAD => {
                let address = read_word(input_reader);
                let size = read_word(input_reader);
                if options.verbose {
                    println!("Section Load");
                    println!("Address: 0x{address:X}");
                    println!("Size: {size} bytes");
                }
                lowest_address = min(lowest_address, address);
                let mut bytes_left = size as usize;
                let mut section_data: Vec<u8> = vec![];
                let mut buffer = [0u8; 4096];
                while bytes_left > 0 {
                    let read_size = min(bytes_left, buffer.len());
                    input_reader.read_exact(&mut buffer[0..read_size])?;
                    bytes_left -= read_size;
                    section_data.write(&buffer[0..read_size])?;
                }
                sections.push((address, section_data));
            }
            CMD_SECTION_FILL => {
                let address = read_word(input_reader);
                let size = read_word(input_reader);
                let access_type = read_word(input_reader);
                let pattern = read_word(input_reader);
                if options.verbose {
                    println!("Section Fill");
                    println!("Address: 0x{address:X}");
                    println!("Size: {size} bytes");
                    print!("Memory Access Type: ");
                    match access_type {
                        0 => println!("8-bit"),
                        1 => println!("16-bit"),
                        2 => println!("32-bit"),
                        _ => println!("Unknown"),
                    }
                    println!("Pattern: 0x{pattern:X}");
                }
                let mut bytes_left = size as usize;
                let mut section_data: Vec<u8> = vec![];
                while bytes_left > 0 {
                    bytes_left -= 4;
                    section_data.write(&pattern.to_le_bytes())?;
                }
                sections.push((address, section_data));
            }
            CMD_CRC_ENABLE => {
                // Does nothing, TODO
                if options.verbose {
                    println!("Enable CRC");
                }
                if !crc_notified {
                    println!("CRC command found, this converter does not support CRC validation.");
                    crc_notified = true;
                }
            }
            CMD_CRC_DISABLE => {
                // Does nothing, TODO
                if options.verbose {
                    println!("Disable CRC");
                }
            }
            CMD_VALIDATE_CRC => {
                // Does nothing, TODO
                if options.verbose {
                    println!("Validate CRC");
                }
                if !crc_notified {
                    println!("CRC command found, this converter does not support CRC validation.");
                    crc_notified = true;
                }
            }
            CMD_JUMP => {
                /*
                Command only serves to notify the user that
                the Jump command functionality is unsupported.
                */
                let address = read_word(input_reader);
                if options.verbose {
                    println!("Jump");
                    println!("Address: 0x{address:X}");
                }
                if !jump_notified {
                    println!(
                        "Jump command found, please note that this converter cannot execute code."
                    );
                    jump_notified = true;
                }
                println!("Jump to 0x{address:X}");
            }
            CMD_JUMP_CLOSE => {
                let address = read_word(input_reader);
                if options.verbose {
                    println!("Jump & Close");
                    println!("Address: 0x{address:X}");
                }
                entry_point = address;
                break;
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("File contains unsupported command 0x{command:X}"),
                ));
            }
        }
    }

    // Write sections to output file
    if options.verbose {
        println!("Writing sections to output file...");
    }
    let mut output_writer = BufWriter::new(output_file);
    for (address, section_data) in sections {
        let seek_position = {
            if options.compact {
                (address - lowest_address) as u64
            } else {
                address as u64
            }
        };
        output_writer.seek(SeekFrom::Start(seek_position))?;
        output_writer.write_all(section_data.as_slice())?;
        if options.verbose {
            println!(
                "Wrote {} bytes at 0x{seek_position:X} of output file",
                section_data.len()
            );
        }
    }
    output_writer.flush()?;

    Ok((entry_point, lowest_address))
}
