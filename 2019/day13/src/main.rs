extern crate intcode;

use intcode::CpuIo;
use intcode::IntcodeError;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, PartialEq)]
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
    segment_display: Option<i64>,
}

impl ScreenState {
    fn new() -> Self {
        Self {
            draw: WriteNumberState::None,
            screen: HashMap::new(),
            segment_display: None,
        }
    }

    fn draw_screen(&self) {
        let min_x = self.screen.keys().map(|tup| tup.0).min().unwrap();
        let max_x = self.screen.keys().map(|tup| tup.0).max().unwrap();
        let min_y = self.screen.keys().map(|tup| tup.1).min().unwrap();
        let max_y = self.screen.keys().map(|tup| tup.1).max().unwrap();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let tile = self.screen.get(&(x, y)).unwrap_or(&TileType::Empty);
                let ch = match tile {
                    TileType::Empty => ' ',
                    TileType::Wall => 'W',
                    TileType::Block => 'B',
                    TileType::HorizontalPaddle => '-',
                    TileType::Ball => '*',
                };
                print!("{}", ch);
            }
            println!();
        }
    }

    fn get_location_of_tile(&self, tile: TileType) -> (i64, i64) {
        let elements: Vec<&(i64, i64)> = self
            .screen
            .iter()
            .filter_map(|(k, v)| if v == &tile { Some(k) } else { None })
            .collect();

        if elements.len() != 1 {
            panic!(
                "unepexted number of {:?} elements: {}",
                tile,
                elements.len()
            );
        } else {
            **elements.first().unwrap()
        }
    }
}

impl CpuIo for ScreenState {
    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        //clear screen
        print!("\x1b[2J");
        self.draw_screen();
        sleep(Duration::from_millis(0));
        Ok(())
    }

    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        let paddle = self.get_location_of_tile(TileType::HorizontalPaddle);
        let ball = self.get_location_of_tile(TileType::Ball);

        Ok(match ball.0.cmp(&paddle.0) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        })
    }

    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
        self.draw = match self.draw {
            WriteNumberState::None => WriteNumberState::OneReceived(num),
            WriteNumberState::OneReceived(x) => WriteNumberState::TwoReceived(x, num),
            WriteNumberState::TwoReceived(-1, 0) => {
                self.segment_display = Some(num);
                WriteNumberState::None
            }
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

    let mut mem = intcode::parse_program(include_str!("input.txt"))?;
    // "insert coun"
    mem[0] = 2;
    let mut state = ScreenState::new();
    intcode::execute_with_io(&mut mem, &mut state)?;
    assert!(state.draw == WriteNumberState::None);
    println!("part 2: {}", state.segment_display.unwrap());

    Ok(())
}
