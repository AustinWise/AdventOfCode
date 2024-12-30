use std::collections::HashMap;
use std::error::Error;
use std::ops::Add;

use intcode::CpuIo;
use intcode::IntcodeError;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Vec2 {
    fn new(x: i32, y: i32) -> Self {
        Vec2 { x, y }
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Unexplored,
    Wall,
    Open { is_oxygen_system: bool },
}

#[derive(Clone, Copy, Debug)]
enum MoveDirection {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Debug)]
enum PendingMove {
    MovingForward(MoveDirection),
    MovingBack(MoveDirection),
}

impl MoveDirection {
    fn move_command(self) -> i64 {
        match self {
            MoveDirection::North => 1,
            MoveDirection::South => 2,
            MoveDirection::West => 3,
            MoveDirection::East => 4,
        }
    }
    fn opposite(self) -> MoveDirection {
        match self {
            MoveDirection::North => MoveDirection::South,
            MoveDirection::East => MoveDirection::West,
            MoveDirection::South => MoveDirection::North,
            MoveDirection::West => MoveDirection::East,
        }
    }

    fn move_direction(self) -> Vec2 {
        match self {
            MoveDirection::North => Vec2::new(0, -1),
            MoveDirection::East => Vec2::new(1, 0),
            MoveDirection::South => Vec2::new(0, 1),
            MoveDirection::West => Vec2::new(-1, 0),
        }
    }
}

struct Map {
    // cells that don't appear here are "Unexplored"
    cells: HashMap<Vec2, CellState>,
    pos: Vec2,
    pending_moving: Option<PendingMove>,
    past_moves: Vec<MoveDirection>,
    moves: usize,
}

impl CpuIo for Map {
    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        Ok(())
    }

    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        assert!(self.pending_moving.is_none());
        for move_dir in [
            MoveDirection::North,
            MoveDirection::East,
            MoveDirection::South,
            MoveDirection::West,
        ] {
            let new_post = self.pos + move_dir.move_direction();
            if self.get_cell_state(&new_post) == CellState::Unexplored {
                println!(
                    "moving FORWARD {:?} from {:?} to {:?}",
                    move_dir,
                    self.pos,
                    self.pos + move_dir.move_direction()
                );
                self.pending_moving = Some(PendingMove::MovingForward(move_dir));
                return Ok(move_dir.move_command());
            }
        }
        if let Some(move_dir) = self.past_moves.pop() {
            println!(
                "moving back {:?} from {:?} to {:?}",
                move_dir.opposite(),
                self.pos,
                self.pos + move_dir.opposite().move_direction()
            );
            self.pending_moving = Some(PendingMove::MovingBack(move_dir.opposite()));
            return Ok(move_dir.opposite().move_command());
        }
        Err(IntcodeError::UserInitiatedExit)
    }

    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
        let pending_move = self.pending_moving.expect("Should have pending move");
        self.pending_moving = None;
        match (num, pending_move) {
            (0, PendingMove::MovingForward(pending_move)) => {
                println!("hit wall at {:?}", self.pos + pending_move.move_direction());
                self.cells
                    .insert(self.pos + pending_move.move_direction(), CellState::Wall);
            }
            (0, PendingMove::MovingBack(_)) => {
                panic!("Should always be possible to move backwards.")
            }
            (1 | 2, PendingMove::MovingForward(pending_move)) => {
                self.pos = self.pos + pending_move.move_direction();
                println!(
                    "moved to {:?}, found oxygen system: {:?}",
                    self.pos,
                    num == 2
                );
                self.past_moves.push(pending_move);
                self.cells.insert(
                    self.pos,
                    CellState::Open {
                        is_oxygen_system: num == 2,
                    },
                );
            }
            (1 | 2, PendingMove::MovingBack(pending_move)) => {
                println!("moved back to {:?}", self.pos);
                self.pos = self.pos + pending_move.move_direction();
            }
            _ => panic!("unexpected number: {}", num),
        };

        self.moves += 1;
        if self.moves > 100000 {
            // If this panic gets hit, either your input has a bigger map than mine or there is some
            // sort of programming error that causes the program to go into an infinite loop.
            panic!("too many moves");
        }

        Ok(())
    }
}

impl Map {
    fn new() -> Self {
        let pos = Vec2::new(0, 0);
        let mut cells = HashMap::new();
        cells.insert(
            pos,
            CellState::Open {
                is_oxygen_system: false,
            },
        );
        Self {
            cells,
            past_moves: Vec::new(),
            pending_moving: None,
            pos,
            moves: 0,
        }
    }

    fn get_cell_state(&self, vec: &Vec2) -> CellState {
        *self.cells.get(vec).unwrap_or(&CellState::Unexplored)
    }

    fn print_map(&self) {
        let min_x = self.cells.keys().map(|v| v.x).min().unwrap();
        let min_y = self.cells.keys().map(|v| v.y).min().unwrap();
        let max_x = self.cells.keys().map(|v| v.x).max().unwrap();
        let max_y = self.cells.keys().map(|v| v.y).max().unwrap();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let ch = match self.cells.get(&Vec2::new(x, y)) {
                    Some(CellState::Unexplored) => ' ',
                    None => ' ',
                    Some(CellState::Wall) => '#',
                    Some(CellState::Open { is_oxygen_system: false }) => '.',
                    Some(CellState::Open { is_oxygen_system: true }) => 'O',
                };
                print!("{}", ch);
            }
            println!();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = intcode::parse_program(include_str!("input.txt"))?;
    let mut map = Map::new();
    intcode::execute_with_io(&mut mem, &mut map)?;
    println!();
    map.print_map();
    Ok(())
}
