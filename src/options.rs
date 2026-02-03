use std::{env::args, path::PathBuf, process::exit};

pub struct ProgramOptions {
    pub path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub verbose: bool,
    pub compact: bool,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            path: Default::default(),
            output_path: Default::default(),
            verbose: false,
            compact: false,
        }
    }
}

pub fn get_config() -> Option<ProgramOptions> {
    let mut args = args().skip(1);
    let mut config = ProgramOptions::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => help(),
            "-v" | "--verbose" => config.verbose = true,
            "-o" | "--out" => {
                let Some(out_arg) = args.next() else {
                    help();
                    return None;
                };
                config.output_path = Some(PathBuf::from(out_arg));
            }
            "-c" | "--compact" => config.compact = true,
            _ => {
                if arg.starts_with("-") {
                    help();
                    return None;
                } else {
                    config.path = Some(PathBuf::from(arg));
                }
            }
        }
    }

    if config.path.is_none() {
        help();
        return None;
    }

    return Some(config);
}

const HELP_OPTIONS: [(&str, &str); 4] = [
    ("-o, --out", "Output memory into specified file"),
    (
        "-c, --compact",
        "Trims output file into compact form without leading null bytes",
    ),
    ("-v, --verbose", "Show verbose output"),
    ("-h, --help", "Display help"),
];

const HELP_STR: &str = "ais2mem - Converts AIS files into memory binary files
Usage: ais2mem [OPTIONS] [FILE]
Options:";

fn help() {
    println!("{HELP_STR}");
    for option in HELP_OPTIONS {
        println!("  {:<15} {}", option.0, option.1);
    }
    exit(0);
}
