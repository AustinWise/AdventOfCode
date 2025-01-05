use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Shl;

use utils::Direction;
use utils::Vec2;

#[derive(Clone, Copy)]
enum Cell {
    Wall,
    Open,
    Key(char),
    Door,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct KeyBitmap(usize);

impl KeyBitmap {
    fn new(ch: char) -> Self {
        assert!(ch.is_ascii_lowercase());
        let ndx = ch as usize - 'a' as usize;
        Self(1usize.shl(ndx))
    }

    fn remove_key(self, ch: char) -> Self {
        let other = Self::new(ch);
        Self(self.0 & !other.0)
    }

    fn add_key(self, ch: char) -> Self {
        let other = Self::new(ch);
        Self(self.0 | other.0)
    }

    fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl Display for KeyBitmap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#b}", self.0)
    }
}

#[derive(Clone)]
struct Maze {
    start: Vec2,
    cells: Vec<Vec<Cell>>,
    // NOTE: not every key has a door
    door_locations: HashMap<char, Vec2>,
    keys: KeyBitmap,
}

impl Maze {
    fn parse(input: &str) -> Self {
        let lines: Vec<&str> = input
            .split(&['\r', '\n'])
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|l| l.len() == lines[0].len()));

        let mut start = None;
        let mut cells = Vec::new();
        let mut door_locations = HashMap::new();

        let mut keys = KeyBitmap(0);
        for (y, line) in lines.iter().enumerate() {
            let mut cell_row = Vec::new();
            for (x, ch) in line.chars().enumerate() {
                let cell = match ch {
                    '#' => Cell::Wall,
                    '.' => Cell::Open,
                    'a'..='z' => {
                        keys = keys.add_key(ch);
                        Cell::Key(ch)
                    }
                    'A'..='Z' => {
                        let prev = door_locations.insert(ch, Vec2::new(x as i32, y as i32));
                        assert!(prev.is_none());
                        Cell::Door
                    }
                    '@' => {
                        let prev = start.replace(Vec2::new(x as i32, y as i32));
                        assert!(prev.is_none());
                        Cell::Open
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
            door_locations,
            keys,
        }
    }

    fn get_cell(&self, pos: Vec2) -> Cell {
        // All mazes have a wall border, so it should not be possible to read out of bounds from an interior cell
        self.cells[pos.y as usize][pos.x as usize]
    }

    fn get_cell_mut(&mut self, pos: Vec2) -> &mut Cell {
        self.cells
            .get_mut(pos.y as usize)
            .unwrap()
            .get_mut(pos.x as usize)
            .unwrap()
    }

    fn clone_with_door_removed(&self, key: char, key_location: Vec2) -> Self {
        assert!(key.is_ascii_alphabetic() && key.is_ascii_lowercase());

        let mut door_locations = self.door_locations.clone();
        let door_location = door_locations.remove(&key.to_ascii_uppercase());
        let cells = self.cells.clone();
        let keys = self.keys.remove_key(key);

        let mut ret = Self {
            start: key_location,
            cells,
            door_locations,
            keys,
        };

        *ret.get_cell_mut(key_location) = Cell::Open;
        if let Some(door_location) = door_location {
            *ret.get_cell_mut(door_location) = Cell::Open;
        }

        ret
    }
}

#[derive(Clone)]
struct MazeSubSolution {
    moves_so_far: usize,
    start: Vec2,
    maze: Maze,
}

impl MazeSubSolution {
    fn find_more_sub_solutions(&self) -> Vec<MazeSubSolution> {
        if self.maze.keys.is_empty() {
            // base case, all keys have been collected
            return Vec::new();
        }

        let mut seen: HashMap<Vec2, ()> = HashMap::new();
        let mut to_visit: VecDeque<(Vec2, usize)> = VecDeque::new();

        to_visit.push_back((self.start, 0));

        let mut ret = Vec::new();
        while let Some((pos, distance)) = to_visit.pop_front() {
            if seen.insert(pos, ()).is_some() {
                continue;
            }

            match self.maze.get_cell(pos) {
                Cell::Wall | Cell::Door => (),
                Cell::Open => {
                    for dir in Direction::all() {
                        let pos = pos + dir.move_vector();
                        to_visit.push_back((pos, distance + 1));
                    }
                }
                Cell::Key(key) => {
                    let maze = self.maze.clone_with_door_removed(key, pos);
                    ret.push(MazeSubSolution {
                        maze,
                        moves_so_far: self.moves_so_far + distance,
                        start: pos,
                    });
                }
            }
        }

        ret
    }
}

fn find_shortest_path_through_maze(maze: &Maze) -> usize {
    let initial = MazeSubSolution {
        maze: maze.clone(),
        moves_so_far: 0,
        start: maze.start,
    };
    let mut best_sub_solutions: HashMap<(KeyBitmap, Vec2), MazeSubSolution> = HashMap::new();
    best_sub_solutions.insert((initial.maze.keys, initial.start), initial.clone());
    let mut to_process = VecDeque::new();
    to_process.push_back(initial);

    let mut best = usize::MAX;
    while let Some(maze) = to_process.pop_front() {
        // first ensure we have not already found a better solution to get to this point
        if let Some(best) = best_sub_solutions
            .get(&(maze.maze.keys, maze.start))
            .map(|m| m.moves_so_far)
        {
            if best < maze.moves_so_far {
                continue;
            }
        } else {
            panic!("we should have seen the sub solution by now");
        }

        if maze.maze.keys.is_empty() && maze.moves_so_far < best {
            best = maze.moves_so_far;
            continue;
        }

        for sub in maze.find_more_sub_solutions() {
            if let Some(best) = best_sub_solutions
                .get(&(sub.maze.keys, sub.start))
                .map(|m| m.moves_so_far)
            {
                if best <= sub.moves_so_far {
                    continue;
                }
            }
            best_sub_solutions.insert((sub.maze.keys, sub.start), sub.clone());
            to_process.push_back(sub);
        }
    }

    assert_ne!(usize::MAX, best);
    best
}

fn main() {
    let maze = Maze::parse(include_str!("input.txt"));
    println!("part 1: {}", find_shortest_path_through_maze(&maze));
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

    #[test]
    fn test_larger() {
        let input = r#"
########################
#f.D.E.e.C.b.A.@.a.B.c.#
######################.#
#d.....................#
########################
"#;
        let maze = Maze::parse(input);
        assert_eq!(86, find_shortest_path_through_maze(&maze));
    }

    #[test]
    fn test_case_1() {
        let input = r#"
########################
#...............b.C.D.f#
#.######################
#.....@.a.B.c.d.A.e.F.g#
########################
"#;
        let maze = Maze::parse(input);
        assert_eq!(132, find_shortest_path_through_maze(&maze));
    }

    #[test]
    fn test_case_2() {
        let input = r#"
#################
#i.G..c...e..H.p#
########.########
#j.A..b...f..D.o#
########@########
#k.E..a...g..B.n#
########.########
#l.F..d...h..C.m#
#################
    "#;
        let maze = Maze::parse(input);
        assert_eq!(136, find_shortest_path_through_maze(&maze));
    }

    #[test]
    fn test_case_3() {
        let input = r#"
########################
#@..............ac.GI.b#
###d#e#f################
###A#B#C################
###g#h#i################
########################
"#;
        let maze = Maze::parse(input);
        assert_eq!(81, find_shortest_path_through_maze(&maze));
    }
}
