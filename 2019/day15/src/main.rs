use std::collections::HashMap;
use std::error::Error;

use intcode::CpuIo;
use intcode::IntcodeError;
use utils::Vec2;

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
    fn all() -> [MoveDirection; 4] {
        [
            MoveDirection::North,
            MoveDirection::East,
            MoveDirection::South,
            MoveDirection::West,
        ]
    }

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
    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        assert!(self.pending_moving.is_none());
        for move_dir in MoveDirection::all() {
            let new_post = self.pos + move_dir.move_direction();
            if self.get_cell_state(&new_post) == CellState::Unexplored {
                self.pending_moving = Some(PendingMove::MovingForward(move_dir));
                return Ok(move_dir.move_command());
            }
        }
        if let Some(move_dir) = self.past_moves.pop() {
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
                self.cells
                    .insert(self.pos + pending_move.move_direction(), CellState::Wall);
            }
            (0, PendingMove::MovingBack(_)) => {
                panic!("Should always be possible to move backwards.")
            }
            (1 | 2, PendingMove::MovingForward(pending_move)) => {
                self.pos = self.pos + pending_move.move_direction();
                self.past_moves.push(pending_move);
                self.cells.insert(
                    self.pos,
                    CellState::Open {
                        is_oxygen_system: num == 2,
                    },
                );
            }
            (1 | 2, PendingMove::MovingBack(pending_move)) => {
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
                let ch = match self
                    .cells
                    .get(&Vec2::new(x, y))
                    .unwrap_or(&CellState::Unexplored)
                {
                    CellState::Unexplored => ' ',
                    CellState::Wall => '#',
                    CellState::Open {
                        is_oxygen_system: false,
                    } => '.',
                    CellState::Open {
                        is_oxygen_system: true,
                    } => 'O',
                };
                print!("{}", ch);
            }
            println!();
        }
    }

    // Rather than using something fancy like A*, just try every possible path.
    fn try_find_distance_to_oxygen_system(
        &self,
        next: Vec2,
        goal: Vec2,
        prev: &mut Vec<Vec2>,
        prev_map: &mut HashMap<Vec2, ()>,
    ) -> Option<usize> {
        if prev_map.contains_key(&next) {
            return None;
        }

        match self.get_cell_state(&next) {
            CellState::Open {
                is_oxygen_system: _,
            } => {
                if next == goal {
                    return Some(prev.len());
                }

                prev.push(next);
                prev_map.insert(next, ());

                let mut best: Option<usize> = None;
                for move_dir in MoveDirection::all() {
                    best = match (
                        best,
                        self.try_find_distance_to_oxygen_system(
                            next + move_dir.move_direction(),
                            goal,
                            prev,
                            prev_map,
                        ),
                    ) {
                        (None, best) => best,
                        (best, None) => best,
                        (Some(x), Some(y)) => {
                            if x > y {
                                Some(y)
                            } else {
                                Some(x)
                            }
                        }
                    }
                }

                prev.pop().unwrap();
                prev_map.remove(&next).unwrap();

                best
            }
            _ => None,
        }
    }

    fn find_distance_to_oxygen_system(&self) -> usize {
        let goal = *self
            .cells
            .iter()
            .filter(|(_, v)| {
                **v == CellState::Open {
                    is_oxygen_system: true,
                }
            })
            .map(|(k, _)| k)
            .next()
            .unwrap();

        let mut prev = vec![];
        let mut prev_map = HashMap::new();
        self.try_find_distance_to_oxygen_system(Vec2::new(0, 0), goal, &mut prev, &mut prev_map)
            .unwrap()
    }

    fn find_minutes_required_for_oxygen_to_propagate(&self) -> usize {
        let mut map = self.cells.clone();
        let mut next_map = map.clone();
        let mut count: usize = 0;

        loop {
            let mut found_oxygen_free_zone = false;
            for (&k, &v) in map.iter() {
                if let (
                    k,
                    // We reuse "is_oxygen_system" to indicate whether or not a cell has oxygen,
                    // not just the system.
                    CellState::Open {
                        is_oxygen_system: false,
                    },
                ) = (k, v)
                {
                    found_oxygen_free_zone = true;

                    for dir in MoveDirection::all() {
                        if let Some(CellState::Open {
                            is_oxygen_system: true,
                        }) = map.get(&(k + dir.move_direction()))
                        {
                            next_map.insert(
                                k,
                                CellState::Open {
                                    is_oxygen_system: true,
                                },
                            );
                        }
                    }
                }
            }

            if !found_oxygen_free_zone {
                return count;
            }

            map = next_map.clone();
            count += 1;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = intcode::parse_program(include_str!("input.txt"))?;
    let mut map = Map::new();
    intcode::execute_with_io(&mut mem, &mut map)?;
    assert!(map.cells.len() > 2);
    println!();
    map.print_map();
    println!();
    println!("part 1: {}", map.find_distance_to_oxygen_system());
    println!(
        "part 2: {}",
        map.find_minutes_required_for_oxygen_to_propagate()
    );
    Ok(())
}
