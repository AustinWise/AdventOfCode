use std::error::Error;

use intcode::CpuIo;
use intcode::IntcodeError;

struct Part1State {
    numbers_to_return: Vec<i64>,
    result: Option<i64>,
}

impl CpuIo for Part1State {
    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        Ok(self.numbers_to_return.pop().unwrap())
    }

    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
        let other = self.result.replace(num);
        assert!(other.is_none());
        Ok(())
    }
}

fn part_1(mem: &[i64]) -> usize {
    let mut ret = 0;
    for y in 0..50 {
        for x in 0..50 {
            let mut state = Part1State {
                numbers_to_return: vec![y, x],
                result: None,
            };
            intcode::execute_with_io(&mut mem.to_owned(), &mut state).unwrap();
            ret += match state.result.unwrap() {
                0 => {print!("."); 0},
                1 => {print!("#");1},
                _ => panic!("unexpected"),
            }
        }
        println!();
    }
    ret
}

fn main() -> Result<(), Box<dyn Error>> {
    let mem = intcode::parse_program(include_str!("input.txt"))?;
    println!("part 1: {}", part_1(&mem));
    Ok(())
}
