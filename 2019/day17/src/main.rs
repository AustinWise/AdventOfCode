use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;

use intcode::AsciiCpuIo;
use intcode::IntcodeError;
use utils::Vec2;
use utils::Direction;

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
enum Cell {
    Scaffold,
    RobotOnScaffold(Direction),
    LostRobot,
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Cell::Scaffold => '#',
            Cell::RobotOnScaffold(Direction::Up) => '^',
            Cell::RobotOnScaffold(Direction::Right) => '>',
            Cell::RobotOnScaffold(Direction::Down) => 'V',
            Cell::RobotOnScaffold(Direction::Left) => '<',
            Cell::LostRobot => 'X',
        })
    }
}

struct Map {
    cells: HashMap<Vec2, Cell>,
    width: i32,
    height: i32,
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
                            '^' => Cell::RobotOnScaffold(Direction::Up),
                            'V' => Cell::RobotOnScaffold(Direction::Down),
                            '<' => Cell::RobotOnScaffold(Direction::Left),
                            '>' => Cell::RobotOnScaffold(Direction::Right),
                            'X' => Cell::LostRobot,
                            _ => panic!("unexpected character: {}", ch),
                        },
                    );
                    assert!(prev.is_none());
                }
            }
        }

        let width = lines[0].len() as i32;
        let height = lines.len() as i32;

        Self {
            cells,
            width,
            height,
        }
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

    fn print(&self) {
        for y in 0..=self.height {
            for x in 0..=self.width {
                if let Some(cell) = self.cells.get(&Vec2::new(x, y)) {
                    print!("{}", cell);
                } else {
                    print!(".");
                }
            }
            println!();
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Command {
    TurnRight,
    TurnLeft,
    Move(usize),
}

impl Command {
    fn ascii_size(&self) -> usize {
        match self {
            Command::TurnLeft | Command::TurnRight => 1,
            Command::Move(amount) => f64::log10(*amount as f64).floor() as usize + 1,
        }
    }

    fn ascii(&self) -> String {
        match self {
            Command::TurnLeft => "L".to_owned(),
            Command::TurnRight => "R".to_owned(),
            Command::Move(amount) => amount.to_string(),
        }
    }
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

    let start: Vec<_> = map
        .cells
        .iter()
        .filter(|(_, cell)| matches!(cell, Cell::RobotOnScaffold(_)))
        .collect();
    assert_eq!(1, start.len());
    let start = *start[0].0;

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

#[derive(Debug)]
struct Dictionary {
    entries_by_index: Vec<Vec<Command>>,
    entries_by_value: HashMap<Vec<Command>, usize>,
}

const MIN_FUNCTION_SIZE: usize = 4;
// it is not possible for a function contain more than 10 commands, as that would be 19 characters
// including commas.
const MAX_FUNCTION_SIZE: usize = 10;

fn create_dictionary(commands: &[Command]) -> Dictionary {
    let mut entries_by_index = Vec::new();
    let mut entries_by_value = HashMap::new();

    // TODO: instead of creating a dictionary that contains every "possible" value, construct the dictionary
    // using a single pass over the commands while also compressing.
    for start_ndx in 0..commands.len() {
        for size in MIN_FUNCTION_SIZE..=MAX_FUNCTION_SIZE {
            if start_ndx + size >= commands.len() {
                continue;
            }
            let seq: Vec<Command> = commands[start_ndx..start_ndx + size].to_vec();
            if seq.iter().map(Command::ascii_size).sum::<usize>() + seq.len() - 1 > 20 {
                continue;
            }
            entries_by_value.entry(seq.clone()).or_insert_with(|| {
                let ndx = entries_by_index.len();
                entries_by_index.push(seq);
                ndx
            });
        }
    }

    Dictionary {
        entries_by_index,
        entries_by_value,
    }
}

#[derive(Debug)]
struct FunctionDefinition(Vec<Command>);

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let strs: Vec<_> = self.0.iter().map(Command::ascii).collect();
        write!(f, "{}", strs.join(","))
    }
}

#[derive(Debug)]
enum FunctionReference {
    A,
    B,
    C,
}

#[derive(Debug)]
struct CompressedCommands {
    main_routine: Vec<FunctionReference>,
    function_a: FunctionDefinition,
    function_b: FunctionDefinition,
    function_c: FunctionDefinition,
}

impl CompressedCommands {
    fn format_main_routine(&self) -> String {
        let strs: Vec<_> = self
            .main_routine
            .iter()
            .map(|f| format!("{:?}", f))
            .collect();
        strs.join(",")
    }

    fn reconstruct_commands(&self) -> Vec<Command> {
        self.main_routine
            .iter()
            .flat_map(|f| match f {
                FunctionReference::A => self.function_a.0.clone(),
                FunctionReference::B => self.function_b.0.clone(),
                FunctionReference::C => self.function_c.0.clone(),
            })
            .collect()
    }
}

fn compress_inner(
    commands: &Vec<Command>,
    dic: &Dictionary,
    ndx: usize,
    compressed: Vec<usize>,
) -> Option<CompressedCommands> {
    for len in (MIN_FUNCTION_SIZE..=MAX_FUNCTION_SIZE).rev() {
        if ndx + len > commands.len() {
            continue;
        }

        let seq = commands[ndx..ndx + len].to_vec();
        assert!(seq.len() == len);
        let function = dic.entries_by_value.get(&seq);
        if function.is_none() {
            // the function probably had too long of an ascii value
            continue;
        }
        let function = *function.unwrap();

        let mut compressed = compressed.clone();
        compressed.push(function);

        let mut distinct_values = HashMap::new();
        for &x in compressed.iter() {
            distinct_values.insert(x, ());
        }

        if distinct_values.len() > 3 {
            continue;
        }

        if ndx + len == commands.len() {
            let abc: Vec<usize> = distinct_values.keys().copied().collect();
            assert_eq!(3, abc.len());
            let main_routine = compressed
                .iter()
                .map(|&ndx| {
                    if ndx == abc[0] {
                        FunctionReference::A
                    } else if ndx == abc[1] {
                        FunctionReference::B
                    } else if ndx == abc[2] {
                        FunctionReference::C
                    } else {
                        panic!("unexpected");
                    }
                })
                .collect();
            // we got to the end
            return Some(CompressedCommands {
                main_routine,
                function_a: FunctionDefinition(dic.entries_by_index[abc[0]].clone()),
                function_b: FunctionDefinition(dic.entries_by_index[abc[1]].clone()),
                function_c: FunctionDefinition(dic.entries_by_index[abc[2]].clone()),
            });
        } else {
            // still more to look for
            let final_result = compress_inner(commands, dic, ndx + len, compressed);
            if final_result.is_some() {
                return final_result;
            }
        }
    }

    None
}

fn compress(commands: &Vec<Command>, dic: &Dictionary) -> CompressedCommands {
    compress_inner(commands, dic, 0, vec![]).unwrap()
}

struct CannedAsciiIo {
    lines: Vec<String>,
    ndx: usize,
}

impl AsciiCpuIo for CannedAsciiIo {
    fn get_input_line_for_program(&mut self) -> Result<String, IntcodeError> {
        let ndx = self.ndx;
        self.ndx += 1;
        Ok(self.lines.get(ndx).expect("not enough lines").clone())
    }

    fn accept_output_line_from_program(&mut self, _output: &str) -> Result<(), IntcodeError> {
        // don't print anything
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = intcode::parse_program(include_str!("input.txt"))?;
    let mut state = CollectAscii::new();
    intcode::execute_with_ascii_io(&mut mem, &mut state)?;
    let map = Map::create_from_lines(&state.lines);
    map.print();
    println!("part 1: {}", part_1_alignment_parameters(&map));

    let commands = make_command_list(&map);
    let dic = create_dictionary(&commands);
    let compressed: CompressedCommands = compress(&commands, &dic);
    assert_eq!(commands, compressed.reconstruct_commands());

    println!();
    println!("main routine: {}", compressed.format_main_routine());
    println!("function a: {}", compressed.function_a);
    println!("function b: {}", compressed.function_b);
    println!("function c: {}", compressed.function_c);
    println!();

    let mut mem = intcode::parse_program(include_str!("input.txt"))?;
    mem[0] = 2;

    if std::env::args().any(|s| s == "--interactive") {
        println!();
        println!("(enter the above at the prompts)");
        println!();
        println!();
        println!("Starting ASCII program in move-robot mode");
        println!();

        let part_2 = intcode::execute_with_std_ascii_io(&mut mem)?;
        println!();
        println!("part 2 answer: {}", part_2.unwrap());
    } else {
        println!("running robot-move program non-interactively. Rerun this program with \"--interactive\" command line argument to connect the Intcode robot move program to standard input/output");
        println!();

        let mut state = CannedAsciiIo {
            ndx: 0,
            lines: vec![
                compressed.format_main_routine(),
                format!("{}", compressed.function_a),
                format!("{}", compressed.function_b),
                format!("{}", compressed.function_c),
                "n".to_owned(),
            ],
        };
        println!(
            "part 2 answer: {}",
            intcode::execute_with_ascii_io(&mut mem, &mut state)?.unwrap()
        );
    }

    // TODO: maybe create some sort of GUI that displays the continuous video feed?

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let input = r#"
..#..........
..#..........
#######...###
#.#...#...#.#
#############
..#...#...#..
..#####...^..
"#;
        let lines: Vec<String> = input.lines().map(|s| s.to_owned()).collect();
        let map = Map::create_from_lines(&lines);
        assert_eq!(76, part_1_alignment_parameters(&map));
    }

    #[test]
    fn test_command_ascii_size() {
        assert_eq!(1, Command::TurnLeft.ascii_size());
        assert_eq!(1, Command::TurnRight.ascii_size());
        assert_eq!(1, Command::Move(1).ascii_size());
        assert_eq!(1, Command::Move(9).ascii_size());
        assert_eq!(2, Command::Move(10).ascii_size());
        assert_eq!(2, Command::Move(99).ascii_size());
        assert_eq!(3, Command::Move(100).ascii_size());
    }
}
