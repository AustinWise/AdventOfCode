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

#[derive(Clone, Copy, PartialEq, Eq)]
enum RobotDirection {
    Up,
    Down,
    Left,
    Right,
}

impl RobotDirection {
    fn move_vector(self) -> Vec2 {
        match self {
            RobotDirection::Up => Vec2::new(0, -1),
            RobotDirection::Down => Vec2::new(0, 1),
            RobotDirection::Left => Vec2::new(-1, 0),
            RobotDirection::Right => Vec2::new(1, 0),
        }
    }

    fn turn_right(self) -> Self {
        match self {
            RobotDirection::Up => RobotDirection::Right,
            RobotDirection::Right => RobotDirection::Down,
            RobotDirection::Down => RobotDirection::Left,
            RobotDirection::Left => RobotDirection::Up,
        }
    }

    fn turn_left(self) -> Self {
        match self {
            RobotDirection::Up => RobotDirection::Left,
            RobotDirection::Left => RobotDirection::Down,
            RobotDirection::Down => RobotDirection::Right,
            RobotDirection::Right => RobotDirection::Up,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    Scaffold,
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

    fn get_nearby_cells(&self, cell: Vec2) -> Vec<(Vec2, Cell)> {
        let mut ret = Vec::new();

        let near_loc = Vec2::new(cell.x - 1, cell.y);
        if let Some(&near) = self.cells.get(&near_loc) {
            ret.push((near_loc, near));
        }

        let near_loc = Vec2::new(cell.x + 1, cell.y);
        if let Some(&near) = self.cells.get(&near_loc) {
            ret.push((near_loc, near));
        }

        let near_loc = Vec2::new(cell.x, cell.y - 1);
        if let Some(&near) = self.cells.get(&near_loc) {
            ret.push((near_loc, near));
        }

        let near_loc = Vec2::new(cell.x, cell.y + 1);
        if let Some(&near) = self.cells.get(&near_loc) {
            ret.push((near_loc, near));
        }

        ret
    }
}

fn part_1_alignment_parameters(map: &Map) -> i32 {
    let mut ret = 0;
    for &cell in map.cells.keys() {
        if map
            .get_nearby_cells(cell)
            .iter()
            .filter(|(_, cell)| *cell == Cell::Scaffold)
            .count()
            == 4
        {
            ret += cell.x * cell.y;
        }
    }
    ret
}

#[derive(Debug)]
enum Command {
    TurnRight,
    TurnLeft,
    #[allow(dead_code)]
    Move(usize),
}

fn make_command_list(map: &Map) -> Vec<Command> {
    // Find the end, the only scaffold that has only one neighbor scaffold.
    let end: Vec<_> = map
        .cells
        .iter()
        .filter(|(pos, cell)| **cell == Cell::Scaffold && map.get_nearby_cells(**pos).len() == 1)
        .collect();
    assert_eq!(1, end.len());
    let end = *end[0].0;
    println!("end: {:?}", end);

    let start: Vec<_> = map
        .cells
        .iter()
        .filter(|(_, cell)| matches!(cell, Cell::RobotOnScaffold(_)))
        .collect();
    assert_eq!(1, start.len());
    let start = *start[0].0;
    println!("start: {:?}", start);

    let mut unvisited_locations: HashMap<Vec2, ()> = HashMap::new();
    for (pos, cell) in map.cells.iter() {
        if *cell != Cell::LostRobot {
            unvisited_locations.insert(*pos, ());
        }
    }
    unvisited_locations.remove(&start).unwrap();

    let mut cur_pos = start;
    let mut cur_direction = match map.cells.get(&start).unwrap() {
        Cell::RobotOnScaffold(dir) => *dir,
        _ => panic!("unexpected cell type at start"),
    };
    let mut ret = Vec::new();
    while !unvisited_locations.is_empty() {
        if unvisited_locations.contains_key(&(cur_pos + cur_direction.move_vector())) {
            let move_vec = cur_direction.move_vector();
            let mut move_count: usize = 0;
            loop {
                let next = cur_pos + move_vec;
                if map.cells.contains_key(&next) {
                    move_count += 1;
                    cur_pos = next;
                    unvisited_locations.remove(&next);
                } else {
                    break;
                }
            }
            assert!(move_count > 0);
            ret.push(Command::Move(move_count));
        } else if unvisited_locations
            .contains_key(&(cur_pos + cur_direction.turn_left().move_vector()))
        {
            ret.push(Command::TurnLeft);
            cur_direction = cur_direction.turn_left();
        } else if unvisited_locations
            .contains_key(&(cur_pos + cur_direction.turn_right().move_vector()))
        {
            ret.push(Command::TurnRight);
            cur_direction = cur_direction.turn_right();
        } else {
            // A nice to have would be more complex path finding. But our puzzle does not need it.
            panic!("advanced path finding not implemented");
        }
    }

    assert_eq!(cur_pos, end);
    assert!(unvisited_locations.is_empty());

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

    println!("part 2 commands: {:?}", make_command_list(&map));

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
