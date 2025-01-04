use std::collections::{HashMap, VecDeque};

use utils::Vec2;

enum Cell {
    Start,
    Wall,
    Open,
    Key(char),
    Door(char),
}

struct Maze {
    start: Vec2,
    cells: Vec<Vec<Cell>>,
    door_location: HashMap<char, Vec2>,
}

impl Maze {
    fn parse(input: &str) -> Self {
        let lines: Vec<&str> = input
            .split(&['\r', '\n'])
            .filter(|l| !l.is_empty())
            .collect();
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|l| l.len() == lines[0].len()));

        let mut start = None;
        let mut cells = Vec::new();
        let mut door_location = HashMap::new();

        for (y, line) in lines.iter().enumerate() {
            let mut cell_row = Vec::new();
            for (x, ch) in line.chars().enumerate() {
                let cell = match ch {
                    '#' => Cell::Wall,
                    '.' => Cell::Open,
                    'a'..'z' => Cell::Key(ch),
                    'A'..'Z' => {
                        let prev = door_location.insert(ch, Vec2::new(x as i32, y as i32));
                        assert!(prev.is_none());
                        Cell::Door(ch)
                    }
                    '@' => {
                        let prev = start.replace(Vec2::new(x as i32, y as i32));
                        assert!(prev.is_none());
                        Cell::Start
                    }
                    _ => panic!("Unexpected char at ({}, {}): {}", x, y, ch),
                };
                cell_row.push(cell);
            }
            cells.push(cell_row);
        }

        Maze {
            start: start.unwrap(),
            cells,
            door_location,
        }
    }

    fn find_shortest_path_through_maze(self: &Self, moves_so_far: usize) -> usize {
        let mut seen: HashMap<Vec2, ()> = HashMap::new();
        let mut keys : HashMap<char, (Vec2, usize)> = HashMap::new();
        let mut to_visit : VecDeque<(Vec2,usize)> = VecDeque::new();

        to_visit.push_back((self.start, 0));

        while !to_visit.is_empty() {
            // TODO: breadth first search
        }

        todo!();
    }
}

fn find_shortest_path_through_maze(maze: &Maze) -> usize {
    maze.find_shortest_path_through_maze(0)
}

fn main() {
    let maze = Maze::parse(include_str!("input.txt"));
    println!("part 1: {}", find_shortest_path_through_maze(&maze));
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    static SIMPLE: &str = r#"
#########
#b.A.@.a#
#########
"#;

    #[test]
    fn test_parse() {
        let maze = Maze::parse(SIMPLE);
        assert_eq!(Vec2::new(5, 1), maze.start);
    }

    #[test]
    fn test_simple() {
        let maze = Maze::parse(SIMPLE);
        assert_eq!(8, find_shortest_path_through_maze(&maze));
    }
}
