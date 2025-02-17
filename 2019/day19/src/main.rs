use std::error::Error;

use intcode::CpuIo;
use intcode::IntcodeError;
use intcode::Memory;
use utils::Vec2;

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

fn in_tractor_beam(mem: &Memory, point: Vec2) -> bool {
    let mut state = Part1State {
        numbers_to_return: vec![point.y.into(), point.x.into()],
        result: None,
    };
    intcode::execute_with_io(&mut mem.to_owned(), &mut state).unwrap();
    match state.result.unwrap() {
        0 => false,
        1 => true,
        _ => panic!("unexpected"),
    }
}

fn part_1(mem: &Memory) -> usize {
    let mut ret = 0;
    for y in 0..50 {
        for x in 0..50 {
            if in_tractor_beam(mem, Vec2::new(y, x)) {
                print!("#");
                ret += 1;
            } else {
                print!(".");
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
