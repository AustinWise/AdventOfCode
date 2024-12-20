use std::cmp::min;
use std::collections::hash_map::HashMap;
use std::error::Error;
use std::fmt;
use std::str::Lines;

#[derive(Debug)]
enum MyError {
    LineParseError,
    NoLine,
}

impl Error for MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::LineParseError => write!(f, "failed to parse movement spec"),
            MyError::NoLine => write!(f, "no more lines to parse"),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Movement {
    amount: u32,
    delta: (i32, i32),
}

fn get_move_for_spec(movement: &str) -> Result<Movement, MyError> {
    let mut char_iter = movement.char_indices();
    let delta = if let Some((0, dir)) = char_iter.next() {
        match dir {
            'U' => (0, 1),
            'D' => (0, -1),
            'L' => (-1, 0),
            'R' => (1, 0),
            _ => {
                return Err(MyError::LineParseError);
            }
        }
    } else {
        return Err(MyError::LineParseError);
    };
    let amount = if let Some((split_index, _)) = char_iter.next() {
        if let Ok(amount) = movement.split_at(split_index).1.parse::<u32>() {
            amount
        } else {
            return Err(MyError::LineParseError);
        }
    } else {
        return Err(MyError::LineParseError);
    };

    if amount == 0 {
        return Err(MyError::LineParseError);
    }

    Ok(Movement { amount, delta })
}

//this will not set the value at (0, 0)
fn parse_wire_location(path: &str) -> Result<HashMap<(i32, i32), u32>, MyError> {
    let mut cur_x = 0;
    let mut cur_y = 0;
    let mut cur_length = 0;

    let mut ret = HashMap::new();

    for movement in path.split(',') {
        let spec = get_move_for_spec(movement)?;

        for _ in 0..spec.amount {
            cur_x += spec.delta.0;
            cur_y += spec.delta.1;
            cur_length += 1;

            let key = (cur_x, cur_y);
            ret.insert(key, cur_length);
        }
    }
    Ok(ret)
}

fn parse_next_wire_location(lines: &mut Lines) -> Result<HashMap<(i32, i32), u32>, MyError> {
    if let Some(line) = lines.next() {
        parse_wire_location(line)
    } else {
        Err(MyError::NoLine)
    }
}

fn find_closest_intersection_by_manhattan(
    map1: &HashMap<(i32, i32), u32>,
    map2: &HashMap<(i32, i32), u32>,
) -> Option<i32> {
    let mut ret = None;
    for loc in map1.keys() {
        if map2.get(loc).is_some() {
            let dist = loc.0.abs() + loc.1.abs();
            ret = if let Some(best) = ret {
                Some(min(dist, best))
            } else {
                Some(dist)
            };
        }
    }
    ret
}

fn find_closest_intersection_by_wire_length(
    map1: &HashMap<(i32, i32), u32>,
    map2: &HashMap<(i32, i32), u32>,
) -> Option<u32> {
    let mut ret: Option<u32> = None;
    for (loc, length1) in map1 {
        if let Some(length2) = map2.get(loc) {
            let dist = length1 + length2;
            ret = if let Some(best) = ret {
                Some(min(dist, best))
            } else {
                Some(dist)
            };
        }
    }
    ret
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_contents = std::fs::read_to_string("input.txt")?;
    let mut lines = file_contents.lines();
    let wire1 = parse_next_wire_location(&mut lines)?;
    let wire2 = parse_next_wire_location(&mut lines)?;
    if let Some(dist) = find_closest_intersection_by_manhattan(&wire1, &wire2) {
        println!("found Manhattan dist: {}", dist);
    } else {
        println!("failed to find Manhattan dist");
    }
    if let Some(dist) = find_closest_intersection_by_wire_length(&wire1, &wire2) {
        println!("found wire dist: {}", dist);
    } else {
        println!("failed to find wire dist");
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_spec() {
        get_move_for_spec("").expect_err("parse failed to fail");
        get_move_for_spec("R").expect_err("parse failed to fail");
        get_move_for_spec("Z2").expect_err("parse failed to fail");
        get_move_for_spec("RR").expect_err("parse failed to fail");

        assert_eq!(
            get_move_for_spec("R3").expect("failed to parse"),
            Movement {
                amount: 3,
                delta: (1, 0),
            }
        );
        assert_eq!(
            get_move_for_spec("L33").expect("failed to parse"),
            Movement {
                amount: 33,
                delta: (-1, 0),
            }
        );
        assert_eq!(
            get_move_for_spec("U6").expect("failed to parse"),
            Movement {
                amount: 6,
                delta: (0, 1),
            }
        );
        assert_eq!(
            get_move_for_spec("D333").expect("failed to parse"),
            Movement {
                amount: 333,
                delta: (0, -1),
            }
        );
    }

    #[test]
    fn test_parse_wire_location1() {
        let map1 = parse_wire_location("R8,U5,L5,D3").expect("failed to add");
        let map2 = parse_wire_location("U7,R6,D4,L4").expect("failed to add");
        assert_eq!(
            6,
            find_closest_intersection_by_manhattan(&map1, &map2).expect("failed to find shortest")
        );
        assert_eq!(
            30,
            find_closest_intersection_by_wire_length(&map1, &map2)
                .expect("failed to find shortest")
        );
    }

    #[test]
    fn test_parse_wire_location2() {
        let map1 =
            parse_wire_location("R75,D30,R83,U83,L12,D49,R71,U7,L72").expect("failed to add");
        let map2 = parse_wire_location("U62,R66,U55,R34,D71,R55,D58,R83").expect("failed to add");
        assert_eq!(
            159,
            find_closest_intersection_by_manhattan(&map1, &map2).expect("failed to find shortest")
        );
        assert_eq!(
            610,
            find_closest_intersection_by_wire_length(&map1, &map2)
                .expect("failed to find shortest")
        );
    }

    #[test]
    fn test_parse_wire_location3() {
        let map1 = parse_wire_location("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51")
            .expect("failed to add");
        let map2 =
            parse_wire_location("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7").expect("failed to add");
        assert_eq!(
            135,
            find_closest_intersection_by_manhattan(&map1, &map2).expect("failed to find shortest")
        );
        assert_eq!(
            410,
            find_closest_intersection_by_wire_length(&map1, &map2)
                .expect("failed to find shortest")
        );
    }
}
