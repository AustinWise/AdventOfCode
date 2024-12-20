extern crate intcode;

use intcode::CpuIo;
use intcode::IntcodeError;
use std::collections::HashMap;
use std::error::Error;

#[derive(PartialEq)]
enum TileType {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball,
}

#[derive(PartialEq)]
enum WriteNumberState {
    None,
    OneReceived(i64),
    TwoReceived(i64, i64),
}

struct ScreenState {
    draw: WriteNumberState,
    screen: HashMap<(i64, i64), TileType>,
}

impl ScreenState {
    fn new() -> Self {
        Self {
            draw: WriteNumberState::None,
            screen: HashMap::new(),
        }
    }
}

impl CpuIo for ScreenState {
    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        panic!("unexpected call to read_number");
    }

    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        Ok(())
    }

    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
        self.draw = match self.draw {
            WriteNumberState::None => WriteNumberState::OneReceived(num),
            WriteNumberState::OneReceived(x) => WriteNumberState::TwoReceived(x, num),
            WriteNumberState::TwoReceived(x, y) => {
                self.screen.insert(
                    (x, y),
                    match num {
                        0 => TileType::Empty,
                        1 => TileType::Wall,
                        2 => TileType::Block,
                        3 => TileType::HorizontalPaddle,
                        4 => TileType::Ball,
                        _ => panic!("unexpected tile type: {}", num),
                    },
                );
                WriteNumberState::None
            }
        };
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = intcode::parse_program(include_str!("input.txt"))?;
    let mut state = ScreenState::new();
    intcode::execute_with_io(&mut mem, &mut state)?;
    assert!(state.draw == WriteNumberState::None);
    println!(
        "parse 1: {}",
        state
            .screen
            .values()
            .filter(|v| **v == TileType::Block)
            .count()
    );

    Ok(())
}
