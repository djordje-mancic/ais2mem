use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write, stdin, stdout},
    path::PathBuf,
    process::exit,
};

use crate::{ais::{convert, is_ais}, options::get_config};

mod options;
mod ais;

fn main() {
    let options = get_config().expect("Error retrieving options");

    let Some(path) = &options.path else {
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
    if !is_ais(&mut input_file_reader) {
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

    // Convert AIS input file and write to output file
    convert(&options, &mut input_file_reader, &mut output_file_writer);
}
