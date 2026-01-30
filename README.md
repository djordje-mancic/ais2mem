# ais2mem
Converts Texas Instruments AIS (Application Image Script) files into .bin files. The contents of the AIS file are laid out in the binary file the same way they would be loaded into the memory of a system by it's bootloader.

Please note that this converter doesn't support all AIS commands.

## Usage
The command format is
``ais2mem [OPTIONS] [FILE]``

**Example:** ``ais2mem -o MEMORY.bin IMAGE.ais``

For more information, you can display help with ``ais2mem --help``

## Building
To build this program, you will need to have Cargo (the Rust package manager) installed. 

Afterwards, you should be able to build the program by running ``cargo build`` in the main directory.