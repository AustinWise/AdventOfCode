use std::cmp::max;
use std::error::Error;

use owo_colors::OwoColorize;

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

fn find_starting_point_core(mem: &Memory) -> Vec2 {
    for wave_number in 1..20 {
        for y in 0..=wave_number {
            let candidate = Vec2::new(wave_number, y);
            if in_tractor_beam(mem, candidate) {
                return candidate;
            }
        }
        for x in 0..=wave_number {
            let candidate = Vec2::new(x, wave_number);
            if in_tractor_beam(mem, candidate) {
                return candidate;
            }
        }
    }
    panic!("did not find any points");
}

fn find_starting_point(mem: &Memory) -> Vec2 {
    let point = find_starting_point_core(mem);
    assert!(!in_tractor_beam(mem, point + Vec2::new(1, 0)));
    assert!(!in_tractor_beam(mem, point - Vec2::new(1, 0)));
    point
}

#[allow(dead_code)]
fn print_board(mem: &Memory, left_point: Vec2, right_point: Vec2) {
    let max_x = max(left_point.x, right_point.x) + 5;
    let max_y = max(left_point.y, left_point.y) + 5;

    for y in 0..max_y {
        for x in 0..max_x {
            let point = Vec2::new(x, y);
            if in_tractor_beam(mem, point) {
                if point == left_point {
                    if point == right_point {
                        print!("{}", "#".purple());
                    } else {
                        print!("{}", "#".blue());
                    }
                } else if point == right_point {
                    print!("{}", "#".red());
                } else {
                    print!("#");
                }
            } else {
                print!(".");
            }
        }
        println!();
    }
}

fn part_2(mem: &Memory) -> Vec2 {
    let start_point = find_starting_point(mem);

    let mut left_point = start_point;
    let mut right_point = start_point;

    loop {
        let x_diff = right_point.x - left_point.x;
        assert!(x_diff >= -1);
        let y_diff = left_point.y - right_point.y;
        assert!(y_diff >= -1);

        if x_diff == 99 && y_diff == 99 {
            assert!(!in_tractor_beam(mem, left_point + Vec2::new(-1, 0)));
            assert!(!in_tractor_beam(mem, right_point + Vec2::new(1, 0)));
            return Vec2::new(left_point.x, right_point.y);
        }

        if x_diff > y_diff {
            let mut candidate = left_point + Vec2::new(0, 1);
            while !in_tractor_beam(mem, candidate) {
                candidate = candidate + Vec2::new(1, 0);
            }
            left_point = candidate;
        } else {
            let mut candidate = right_point + Vec2::new(1, 0);
            if in_tractor_beam(mem, candidate) {
            } else {
                candidate = right_point + Vec2::new(1, 1);
                assert!(in_tractor_beam(mem, candidate));
            }
            right_point = candidate;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mem = intcode::parse_program(include_str!("input.txt"))?;
    println!("part 1: {}", part_1(&mem));
    let part_2_res = part_2(&mem);
    println!("part 2: {}", part_2_res.x * 10000 + part_2_res.y);
    Ok(())
}
