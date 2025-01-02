use std::error::Error;

use intcode::AsciiCpuIo;
use intcode::IntcodeError;

struct PrintAscii {}

impl AsciiCpuIo for PrintAscii {
    fn get_input_line_for_program(&mut self) -> Result<String, IntcodeError> {
        panic!("not implemented");
    }

    fn accept_output_line_from_program(&mut self, output: &str) -> Result<(), IntcodeError> {
        println!("{}", output);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = intcode::parse_program(include_str!("input.txt"))?;
    let mut state = PrintAscii {};
    intcode::execute_with_ascii_io(&mut mem, &mut state)?;
    Ok(())
}
