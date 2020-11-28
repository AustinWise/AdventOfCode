use std::error::Error;

extern crate intcode;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = intcode::parse_program(&std::fs::read_to_string("input.txt")?)?;
    intcode::execute(
        &mut mem,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
    )?;
    Ok(())
}
