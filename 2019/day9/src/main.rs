use std::error::Error;

extern crate intcode;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = intcode::parse_program(&std::fs::read_to_string("input.txt")?)?;
    intcode::execute_with_std_io(&mut mem)?;
    Ok(())
}
