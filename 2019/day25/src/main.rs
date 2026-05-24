use std::error::Error;

// TODO: play game?

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = intcode::parse_program(include_str!("input.txt"))?;
    let part_1 = intcode::execute_with_std_ascii_io(&mut mem)?;
    println!("result: {:?}", part_1);
    Ok(())
}
