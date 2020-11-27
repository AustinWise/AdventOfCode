use std::error::Error;
use std::fmt;

extern crate intcode;

#[derive(Debug)]
enum MyError {
    AnswerNotFound
}

impl Error for MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::AnswerNotFound => write!(f, "answer not found"),
        }
    }
}

fn part1(source: &[i32]) -> Result<(), intcode::IntcodeError> {
    let mut mem = source.to_owned();
    mem[1] = 12;
    mem[2] = 2;
    intcode::execute_no_io(&mut mem)?;
    println!("part 1: {}", mem[0]);
    Ok(())
}

fn part2(source: &[i32]) -> Result<(), Box<dyn Error>> {
    for noun in 0..100 {
        for verb in 0..100 {
            let mut mem = source.to_owned();
            mem[1] = noun;
            mem[2] = verb;
            intcode::execute_no_io(&mut mem)?;
            if mem[0] == 19_690_720 {
                println!("part2: {}", 100 * noun + verb);
                return Ok(());
            }
        }
    }
    Err(Box::new(MyError::AnswerNotFound))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mem = intcode::parse_program(&std::fs::read_to_string("input.txt")?)?;
    part1(&mem)?;
    part2(&mem)?;
    Ok(())
}
