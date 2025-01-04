use std::{collections::HashMap, error::Error};

use intcode::CpuIo;
use intcode::IntcodeError;
use utils::Direction;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn move_in_direction(self, dir: Direction) -> Self {
        match dir {
            Direction::Up => Position {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => Position {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Right => Position {
                x: self.x + 1,
                y: self.y,
            },
            Direction::Left => Position {
                x: self.x - 1,
                y: self.y,
            },
        }
    }
}

enum CommandReceiveState {
    AwaitingPaintCommand,
    AwaitingTurnCommand,
}

struct State {
    position: Position,
    command_state: CommandReceiveState,
    direction: Direction,
    // true if white, false if black. if not listed, the square is black
    is_painted_white: HashMap<Position, bool>,
}

impl State {
    fn new() -> Self {
        State {
            position: Position { x: 0, y: 0 },
            command_state: CommandReceiveState::AwaitingPaintCommand,
            direction: Direction::Up,
            is_painted_white: HashMap::new(),
        }
    }
}

impl CpuIo for State {
    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        let pos = self.position;
        let color = self.is_painted_white.get(&pos).unwrap_or(&false);
        Ok(*color as i64)
    }

    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
        match self.command_state {
            CommandReceiveState::AwaitingPaintCommand => {
                self.is_painted_white.insert(
                    self.position,
                    match num {
                        0 => false,
                        1 => true,
                        _ => panic!("unexpected paint value"),
                    },
                );
                self.command_state = CommandReceiveState::AwaitingTurnCommand;
            }
            CommandReceiveState::AwaitingTurnCommand => {
                match num {
                    0 => self.direction = self.direction.turn_left(),
                    1 => self.direction = self.direction.turn_right(),
                    _ => panic!("unexpected direction command"),
                }
                self.position = self.position.move_in_direction(self.direction);
                self.command_state = CommandReceiveState::AwaitingPaintCommand;
            }
        }

        Ok(())
    }
    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        Ok(())
    }
}

fn init_computer() -> Result<(Vec<i64>, State), IntcodeError> {
    let mem = intcode::parse_program(&std::fs::read_to_string("input.txt")?)?;
    let state = State::new();
    Ok((mem, state))
}

fn main() -> Result<(), Box<dyn Error>> {
    let (mut mem, mut state) = init_computer()?;
    intcode::execute_with_io(&mut mem, &mut state)?;
    println!(
        "number of panels painted (part1): {}",
        state.is_painted_white.len()
    );

    let (mut mem, mut state) = init_computer()?;
    state.is_painted_white.insert(state.position, true);
    intcode::execute_with_io(&mut mem, &mut state)?;

    let min_x = state
        .is_painted_white
        .keys()
        .map(|pos| pos.x)
        .min()
        .unwrap();
    let max_x = state
        .is_painted_white
        .keys()
        .map(|pos| pos.x)
        .max()
        .unwrap();
    let min_y = state
        .is_painted_white
        .keys()
        .map(|pos| pos.y)
        .min()
        .unwrap();
    let max_y = state
        .is_painted_white
        .keys()
        .map(|pos| pos.y)
        .max()
        .unwrap();

    println!(
        "part2: painted from ({}, {}) to ({}, {})",
        min_x, min_y, max_x, max_y
    );

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            print!(
                "{}",
                match state
                    .is_painted_white
                    .get(&Position { x, y })
                    .unwrap_or(&false)
                {
                    false => ' ',
                    true => '#',
                }
            )
        }
        println!();
    }

    Ok(())
}
