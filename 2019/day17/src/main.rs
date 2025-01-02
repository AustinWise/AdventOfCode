use std::collections::HashMap;
use std::error::Error;

use intcode::AsciiCpuIo;
use intcode::IntcodeError;
use utils::Vec2;

struct CollectAscii {
    lines: Vec<String>,
}

impl CollectAscii {
    fn new() -> Self {
        Self { lines: vec![] }
    }
}

impl AsciiCpuIo for CollectAscii {
    fn get_input_line_for_program(&mut self) -> Result<String, IntcodeError> {
        panic!("not implemented");
    }

    fn accept_output_line_from_program(&mut self, output: &str) -> Result<(), IntcodeError> {
        self.lines.push(output.to_owned());
        Ok(())
    }
}

enum RobotDirection {
    Up,
    Down,
    Left,
    Right,
}

enum Cell {
    Scaffold,
    #[allow(dead_code)]
    RobotOnScaffold(RobotDirection),
    LostRobot,
}

struct Map {
    cells: HashMap<Vec2, Cell>,
}

impl Map {
    fn create_from_lines(lines: &[String]) -> Self {
        // skip empty lines (the program outputs an empty line at the end)
        let lines: Vec<_> = lines.iter().filter(|l| !l.is_empty()).collect();
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|l| l.len() == lines[0].len()));

        let mut cells = HashMap::new();
        for (y, line) in lines.iter().filter(|l| !l.is_empty()).enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch != '.' {
                    let prev = cells.insert(
                        Vec2::new(x as i32, y as i32),
                        match ch {
                            '#' => Cell::Scaffold,
                            '^' => Cell::RobotOnScaffold(RobotDirection::Up),
                            'V' => Cell::RobotOnScaffold(RobotDirection::Down),
                            '<' => Cell::RobotOnScaffold(RobotDirection::Left),
                            '>' => Cell::RobotOnScaffold(RobotDirection::Right),
                            'X' => Cell::LostRobot,
                            _ => panic!("unexpected character: {}", ch),
                        },
                    );
                    assert!(prev.is_none());
                }
            }
        }
        Self { cells }
    }
}

fn part_1_alignment_parameters(map: &Map) -> i32 {
    let mut ret = 0;
    for cell in map.cells.keys() {
        if map.cells.contains_key(&Vec2::new(cell.x - 1, cell.y))
            && map.cells.contains_key(&Vec2::new(cell.x + 1, cell.y))
            && map.cells.contains_key(&Vec2::new(cell.x, cell.y - 1))
            && map.cells.contains_key(&Vec2::new(cell.x, cell.y + 1))
        {
            ret += cell.x * cell.y;
        }
    }
    ret
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = intcode::parse_program(include_str!("input.txt"))?;
    let mut state = CollectAscii::new();
    intcode::execute_with_ascii_io(&mut mem, &mut state)?;
    for line in &state.lines {
        println!("{}", line);
    }
    let map = Map::create_from_lines(&state.lines);
    println!("part 1: {}", part_1_alignment_parameters(&map));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let lines: Vec<String> = r#"..#..........
..#..........
#######...###
#.#...#...#.#
#############
..#...#...#..
..#####...^..
"#
        .lines()
        .map(|s| s.to_owned())
        .collect();
        let map = Map::create_from_lines(&lines);
        assert_eq!(76, part_1_alignment_parameters(&map));
    }
}
