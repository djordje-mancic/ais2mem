use std::{
    cmp::min,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Seek, Write, stdin, stdout},
    path::PathBuf,
    process::exit,
};

use crate::options::get_config;

mod options;

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

fn main() {
    let options = get_config().expect("Error retrieving options");

    let Some(path) = options.path else {
        eprintln!("File path must be specified");
        exit(-1);
    };
    let input_file_result = File::open(&path);
    let Ok(input_file) = input_file_result else {
        let _ = input_file_result.inspect_err(|e| eprintln!("Couldn't open file {:?}: {e}", path));
        exit(-1);
    };
    let mut input_file_reader = BufReader::new(input_file);

    // Check if magic word at the start of file is correct
    if read_word(&mut input_file_reader) != 0x41504954 {
        eprintln!("File specified isn't an Application Image Script file");
        exit(-1);
    }

    // Create output file
    let output_path = {
        if let Some(path_string) = &options.output_path {
            PathBuf::from(path_string)
        } else {
            let mut cloned_path = path.clone();
            cloned_path.set_extension("bin");
            cloned_path
        }
    };

    if output_path.exists() == true {
        loop {
            print!(
                "File at path {:?} already exists, overwrite? (Y/N): ",
                output_path
            );
            let _ = stdout().flush();
            let mut input_line = String::new();
            let _ = stdin().read_line(&mut input_line);
            match input_line.to_uppercase().as_str() {
                "Y\n" => break,
                "N\n" => exit(-1),
                _ => (),
            }
        }
    }

    let output_file_result = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_path);
    let Ok(output_file) = output_file_result else {
        let _ = output_file_result.inspect_err(|e| {
            eprintln!("Couldn't create file {:?}: {e}", path);
            if options.output_path.is_none() {
                eprintln!("You might need to specify the output path with the -o option. More information can be found with 'ais2mem --help'");
            }
        });
        exit(-1);
    };
    let mut output_file_writer = BufWriter::new(output_file);

    // Command read loop
    loop {
        let command = read_word(&mut input_file_reader);
        if options.verbose {
            println!("Command {command:x}");
        }
        match command {
            CMD_SECTION_LOAD => {
                let address = read_word(&mut input_file_reader);
                let size = read_word(&mut input_file_reader);
                if options.verbose {
                    println!("Section Load");
                    println!("Address: {address:x}");
                    println!("Size: {size} bytes");
                }
                let mut bytes_left = size as usize;

                if let Err(e) = output_file_writer.seek(std::io::SeekFrom::Start(address as u64)) {
                    eprintln!("Error seeking to address in output file: {e}");
                    exit(-1);
                }

                const READ_BUFFER_SIZE: usize = 4096;
                let mut buffer = [0u8; READ_BUFFER_SIZE];
                while bytes_left > 0 {
                    let read_size = min(bytes_left, READ_BUFFER_SIZE);
                    if let Err(e) = input_file_reader.read_exact(&mut buffer[0..read_size]) {
                        eprintln!("Error reading file contents: {e}");
                        exit(-1);
                    }
                    if let Err(e) = output_file_writer.write_all(&buffer[0..read_size]) {
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
                let address = read_word(&mut input_file_reader);
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
