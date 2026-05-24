use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;

use itertools::Itertools;

use utils::Direction;
use utils::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RawCell {
    Space,
    Wall,
    Open,
    Portal(char),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct PortalName {
    is_inner: bool,
    first_character: char,
    second_character: char,
}

impl Debug for PortalName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{}",
            if self.is_inner { "Inner" } else { "Outer" },
            self.first_character,
            self.second_character
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    Wall,
    Open,
    Portal(PortalName),
}

struct PortalLocations {
    letter_loc: Vec2,
    adjacent_loc: Vec2,
}

struct Maze {
    grid: Vec<Vec<Cell>>,
    // A map from the letter entry point to the location in front of the other side of the portal.
    portal_map: HashMap<Vec2, Vec2>,
    start: Vec2,
    end: Vec2,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct ShortestPathNode {
    cost: usize,
    level: usize,
    loc: Vec2,
}

impl Ord for ShortestPathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Note that the cost comparison is reversed to make this a min heap.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.level.cmp(&other.level))
            .then_with(|| self.loc.y.cmp(&other.loc.y))
            .then_with(|| self.loc.x.cmp(&other.loc.x))
    }
}

impl PartialOrd for ShortestPathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Maze {
    fn get_cell(&self, loc: Vec2) -> Cell {
        assert!(loc.x >= 0);
        assert!(loc.y >= 0);
        self.grid[loc.y as usize][loc.x as usize]
    }

    // This is Dijkstra’s algorithm for shortest path, basically copy-pasted from the documentation
    // of BinaryHeap: https://doc.rust-lang.org/std/collections/binary_heap/
    fn shortest_path_to_end(&self) -> usize {
        let mut dist: HashMap<Vec2, usize> = HashMap::new();
        let mut heap: BinaryHeap<ShortestPathNode> = BinaryHeap::new();

        dist.insert(self.start, 0);
        heap.push(ShortestPathNode {
            cost: 0,
            level: 0,
            loc: self.start,
        });

        while let Some(ShortestPathNode {
            cost,
            loc,
            level: _,
        }) = heap.pop()
        {
            if loc == self.end {
                return cost;
            }

            if cost > *dist.get(&loc).unwrap_or(&usize::MAX) {
                continue;
            }

            for next in Direction::all().iter().filter_map(|dir| {
                let next_loc = loc + dir.move_vector();
                let next = self.get_cell(next_loc);
                match next {
                    Cell::Open => Some(ShortestPathNode {
                        cost: cost + 1,
                        level: 0,
                        loc: next_loc,
                    }),
                    Cell::Portal(_) => Some(ShortestPathNode {
                        cost: cost + 1,
                        level: 0,
                        loc: *self.portal_map.get(&next_loc).unwrap(),
                    }),
                    Cell::Wall => None,
                }
            }) {
                if next.cost < *dist.get(&next.loc).unwrap_or(&usize::MAX) {
                    heap.push(next);
                    dist.insert(next.loc, next.cost);
                }
            }
        }

        panic!("No path!");
    }

    fn shortest_recursive_path_to_end(&self) -> usize {
        let mut dist: HashMap<(Vec2, usize), usize> = HashMap::new();
        let mut heap: BinaryHeap<ShortestPathNode> = BinaryHeap::new();

        dist.insert((self.start, 0), 0);
        heap.push(ShortestPathNode {
            cost: 0,
            level: 0,
            loc: self.start,
        });

        while let Some(ShortestPathNode { cost, loc, level }) = heap.pop() {
            if loc == self.end && level == 0 {
                return cost;
            }

            if cost > *dist.get(&(loc, level)).unwrap_or(&usize::MAX) {
                continue;
            }

            for next in Direction::all().iter().filter_map(|dir| {
                let next_loc = loc + dir.move_vector();
                let next = self.get_cell(next_loc);
                match next {
                    Cell::Open => Some(ShortestPathNode {
                        cost: cost + 1,
                        level,
                        loc: next_loc,
                    }),
                    Cell::Portal(p) if level == 0 && !p.is_inner => None,
                    Cell::Portal(p) => Some(ShortestPathNode {
                        cost: cost + 1,
                        level: if p.is_inner {
                            level + 1
                        } else {
                            assert!(level > 0);
                            level - 1
                        },
                        loc: *self.portal_map.get(&next_loc).unwrap(),
                    }),
                    Cell::Wall => None,
                }
            }) {
                if next.cost < *dist.get(&(next.loc, next.level)).unwrap_or(&usize::MAX) {
                    heap.push(next);
                    dist.insert((next.loc, next.level), next.cost);
                }
            }
        }

        panic!("No path!");
    }
}

fn load_puzzle(input: &str) -> Maze {
    let mut lines: Vec<Vec<RawCell>> = Vec::new();
    for line in input.split("\n") {
        if line.is_empty() {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        if !lines.is_empty() && chars.len() != lines[0].len() {
            panic!("Line length mismatch.");
        }
        lines.push(
            chars
                .iter()
                .map(|&ch| match ch {
                    ' ' => RawCell::Space,
                    '#' => RawCell::Wall,
                    '.' => RawCell::Open,
                    'A'..='Z' => RawCell::Portal(ch),
                    _ => panic!("Unexpected char: {}", ch),
                })
                .collect(),
        );
    }

    let raw_cells = lines;

    let height = raw_cells.len();
    let width = raw_cells[0].len();
    let bottom_right_corner = Vec2::from_usize_or_panic(width - 1, height - 1);

    let mut grid: Vec<Vec<Cell>> = Vec::new();
    let mut portals: HashMap<(char, char), Vec<PortalLocations>> = HashMap::new();
    let mut start: Option<Vec2> = None;
    let mut end: Option<Vec2> = None;
    for (y, line) in raw_cells.iter().enumerate() {
        let mut grid_line: Vec<Cell> = Vec::new();
        for (x, &raw_cell) in line.iter().enumerate() {
            let cell = match raw_cell {
                RawCell::Space | RawCell::Wall => Cell::Wall,
                RawCell::Open => Cell::Open,
                RawCell::Portal(ch) => {
                    // For letters around the edge, we will handle them when we come around to the
                    // second letter.
                    if y == 0 || x == 0 || y == height - 1 || x == width - 1 {
                        Cell::Wall
                    } else {
                        let loc = Vec2::from_usize_or_panic(x, y);

                        let neighbor_cells: Vec<_> = Direction::all()
                            .iter()
                            .map(|dir| {
                                let neighbor_loc = loc + dir.move_vector();
                                let neighbor_cell =
                                    raw_cells[neighbor_loc.y as usize][neighbor_loc.x as usize];
                                (neighbor_loc, neighbor_cell)
                            })
                            .collect();

                        let neighbor_letter = neighbor_cells
                            .iter()
                            .filter_map(|(neighbor_loc, neighbor_cell)| match neighbor_cell {
                                RawCell::Portal(ch) => Some((*neighbor_loc, *ch)),
                                _ => None,
                            })
                            .at_most_one()
                            .expect("Too many neighbor letters.")
                            .expect("Missing neighbor letter.");

                        let neighbor_open = neighbor_cells
                            .iter()
                            .filter_map(|(neighbor_loc, neighbor_cell)| match neighbor_cell {
                                RawCell::Open => Some(*neighbor_loc),
                                _ => None,
                            })
                            .at_most_one()
                            .expect("Should have at most one neighbor open.");

                        if let Some(neighbor_open) = neighbor_open {
                            let neighbor_letter_loc = neighbor_letter.0;
                            let neighbor_letter_first =
                                neighbor_letter_loc.x <= loc.x && neighbor_letter_loc.y <= loc.y;
                            let first_character = if neighbor_letter_first {
                                neighbor_letter.1
                            } else {
                                ch
                            };
                            let second_character = if !neighbor_letter_first {
                                neighbor_letter.1
                            } else {
                                ch
                            };
                            match (first_character, second_character) {
                                ('A', 'A') => {
                                    assert!(start.is_none());
                                    start = Some(neighbor_open);
                                    Cell::Wall
                                }
                                ('Z', 'Z') => {
                                    assert!(end.is_none());
                                    end = Some(neighbor_open);
                                    Cell::Wall
                                }
                                _ => {
                                    portals
                                        .entry((first_character, second_character))
                                        .or_default()
                                        .push(PortalLocations {
                                            letter_loc: loc,
                                            adjacent_loc: neighbor_open,
                                        });
                                    let is_outer = loc.x == 1
                                        || loc.y == 1
                                        || loc.x == bottom_right_corner.x - 1
                                        || loc.y == bottom_right_corner.y - 1;
                                    Cell::Portal(PortalName {
                                        is_inner: !is_outer,
                                        first_character,
                                        second_character,
                                    })
                                }
                            }
                        } else {
                            // We are the outside letter, away for the teleport location.
                            Cell::Wall
                        }
                    }
                }
            };
            grid_line.push(cell);
        }
        grid.push(grid_line);
    }

    let mut portal_map: HashMap<Vec2, Vec2> = HashMap::new();

    for entry in portals.values() {
        assert!(entry.len() == 2);
        assert!(
            portal_map
                .insert(entry[0].letter_loc, entry[1].adjacent_loc)
                .is_none()
        );
        assert!(
            portal_map
                .insert(entry[1].letter_loc, entry[0].adjacent_loc)
                .is_none()
        );
    }

    Maze {
        grid,
        portal_map,
        start: start.unwrap(),
        end: end.unwrap(),
    }
}

fn main() {
    let maze = load_puzzle(include_str!("input.txt"));
    println!("Part 1: {}", maze.shortest_path_to_end());
    println!("Part 2: {}", maze.shortest_recursive_path_to_end());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_small_example() {
        let input = "
         A           
         A           
  #######.#########  
  #######.........#  
  #######.#######.#  
  #######.#######.#  
  #######.#######.#  
  #####  B    ###.#  
BC...##  C    ###.#  
  ##.##       ###.#  
  ##...DE  F  ###.#  
  #####    G  ###.#  
  #########.#####.#  
DE..#######...###.#  
  #.#########.###.#  
FG..#########.....#  
  ###########.#####  
             Z       
             Z       
";
        let maze = load_puzzle(input);
        assert_eq!(23, maze.shortest_path_to_end());
    }

    #[test]
    fn test_larger_example() {
        let input = "


                   A               
                   A               
  #################.#############  
  #.#...#...................#.#.#  
  #.#.#.###.###.###.#########.#.#  
  #.#.#.......#...#.....#.#.#...#  
  #.#########.###.#####.#.#.###.#  
  #.............#.#.....#.......#  
  ###.###########.###.#####.#.#.#  
  #.....#        A   C    #.#.#.#  
  #######        S   P    #####.#  
  #.#...#                 #......VT
  #.#.#.#                 #.#####  
  #...#.#               YN....#.#  
  #.###.#                 #####.#  
DI....#.#                 #.....#  
  #####.#                 #.###.#  
ZZ......#               QG....#..AS
  ###.###                 #######  
JO..#.#.#                 #.....#  
  #.#.#.#                 ###.#.#  
  #...#..DI             BU....#..LF
  #####.#                 #.#####  
YN......#               VT..#....QG
  #.###.#                 #.###.#  
  #.#...#                 #.....#  
  ###.###    J L     J    #.#.###  
  #.....#    O F     P    #.#...#  
  #.###.#####.#.#####.#####.###.#  
  #...#.#.#...#.....#.....#.#...#  
  #.#####.###.###.#.#.#########.#  
  #...#.#.....#...#.#.#.#.....#.#  
  #.###.#####.###.###.#.#.#######  
  #.#.........#...#.............#  
  #########.###.###.#############  
           B   J   C               
           U   P   P               
";
        let maze = load_puzzle(input);
        assert_eq!(58, maze.shortest_path_to_end());
    }

    #[test]
    fn test_small_example_recursive() {
        let input = "
         A           
         A           
  #######.#########  
  #######.........#  
  #######.#######.#  
  #######.#######.#  
  #######.#######.#  
  #####  B    ###.#  
BC...##  C    ###.#  
  ##.##       ###.#  
  ##...DE  F  ###.#  
  #####    G  ###.#  
  #########.#####.#  
DE..#######...###.#  
  #.#########.###.#  
FG..#########.....#  
  ###########.#####  
             Z       
             Z       
";
        let maze = load_puzzle(input);
        assert_eq!(26, maze.shortest_recursive_path_to_end());
    }

    #[test]
    fn test_larger_example_recursive() {
        let input = "

             Z L X W       C                 
             Z P Q B       K                 
  ###########.#.#.#.#######.###############  
  #...#.......#.#.......#.#.......#.#.#...#  
  ###.#.#.#.#.#.#.#.###.#.#.#######.#.#.###  
  #.#...#.#.#...#.#.#...#...#...#.#.......#  
  #.###.#######.###.###.#.###.###.#.#######  
  #...#.......#.#...#...#.............#...#  
  #.#########.#######.#.#######.#######.###  
  #...#.#    F       R I       Z    #.#.#.#  
  #.###.#    D       E C       H    #.#.#.#  
  #.#...#                           #...#.#  
  #.###.#                           #.###.#  
  #.#....OA                       WB..#.#..ZH
  #.###.#                           #.#.#.#  
CJ......#                           #.....#  
  #######                           #######  
  #.#....CK                         #......IC
  #.###.#                           #.###.#  
  #.....#                           #...#.#  
  ###.###                           #.#.#.#  
XF....#.#                         RF..#.#.#  
  #####.#                           #######  
  #......CJ                       NM..#...#  
  ###.#.#                           #.###.#  
RE....#.#                           #......RF
  ###.###        X   X       L      #.#.#.#  
  #.....#        F   Q       P      #.#.#.#  
  ###.###########.###.#######.#########.###  
  #.....#...#.....#.......#...#.....#.#...#  
  #####.#.###.#######.#######.###.###.#.#.#  
  #.......#.......#.#.#.#.#...#...#...#.#.#  
  #####.###.#####.#.#.#.#.###.###.#.###.###  
  #.......#.....#.#...#...............#...#  
  #############.#.#.###.###################  
               A O F   N                     
               A A D   M                     
";
        let maze = load_puzzle(input);
        assert_eq!(396, maze.shortest_recursive_path_to_end());
    }
}
