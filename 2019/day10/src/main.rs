use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
enum ParseError {
    #[error("unexpected char: {0}")]
    UnexpectedCharacter(char),
    #[error("uneven line length")]
    UnevenLineLength,
    #[error("no lines in input")]
    NoLines,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    #[allow(dead_code)]
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug)]
struct RelativePosition {
    pos: Position,
    angle: OrderedFloat<f64>,
    magnitude: OrderedFloat<f64>,
}

impl RelativePosition {
    fn new(origin: Position, pos: Position) -> Self {
        let x = pos.x - origin.x;
        let y = pos.y - origin.y;
        let angle = OrderedFloat(-1.0 * f64::atan2(x as f64, y as f64));
        let magnitude = OrderedFloat(f64::sqrt((x as f64).powi(2) + (y as f64).powi(2)));
        Self {
            pos,
            angle,
            magnitude,
        }
    }
}

struct AsteroidMap {
    width: i32,
    height: i32,
    asteroids: Vec<Position>,
}

impl AsteroidMap {
    fn parse(input: &str) -> Result<AsteroidMap, ParseError> {
        let mut asteroids: Vec<Position> = Vec::new();
        let mut y = 0;
        let mut first_line_length = None;
        for line in input.split('\n') {
            if !line.is_empty() {
                let mut x = 0;
                for c in line.chars() {
                    match c {
                        '.' => {}
                        '#' => asteroids.push(Position { x, y }),
                        _ => return Err(ParseError::UnexpectedCharacter(c)),
                    };
                    x += 1;
                }
                if let Some(first_line_length) = first_line_length {
                    if first_line_length != x {
                        return Err(ParseError::UnevenLineLength);
                    }
                }
                first_line_length = Some(x);
                y += 1;
            }
        }

        if asteroids.is_empty() {
            return Err(ParseError::NoLines);
        }

        Ok(AsteroidMap {
            width: first_line_length.unwrap(),
            height: y,
            asteroids,
        })
    }
}

struct PlacementResult {
    pos: Position,
    num_detectable: usize,
}

#[derive(Error, Debug)]
enum PlacementError {
    #[error("no asteroids in starfield")]
    NoAsteroids,
}

fn count_asteroids_in_line_of_sight(map: &AsteroidMap, origin: Position) -> PlacementResult {
    let mut seen_angles: HashSet<OrderedFloat<f64>> = HashSet::new();
    for &pos in &map.asteroids {
        if !origin.eq(&pos) {
            let rel = RelativePosition::new(origin, pos);
            seen_angles.insert(rel.angle);
        }
    }
    PlacementResult {
        pos: origin,
        num_detectable: seen_angles.len(),
    }
}

fn get_num_detectable(map: &AsteroidMap) -> Result<PlacementResult, PlacementError> {
    let mut ret: Option<PlacementResult> = None;

    for &pos in map.asteroids.iter() {
        let candidate = count_asteroids_in_line_of_sight(map, pos);
        if ret.is_none() || ret.as_ref().unwrap().num_detectable < candidate.num_detectable {
            ret = Some(candidate);
        }
    }

    if let Some(ret) = ret {
        Ok(ret)
    } else {
        Err(PlacementError::NoAsteroids)
    }
}

fn generate_laser_firing_sequence(map: &AsteroidMap, place: &PlacementResult) -> Vec<Position> {
    let mut grouped: HashMap<OrderedFloat<f64>, Vec<RelativePosition>> = HashMap::new();
    for &pos in map.asteroids.iter() {
        if !pos.eq(&place.pos) {
            let rel = RelativePosition::new(place.pos, pos);
            if let Some(existing) = grouped.get_mut(&rel.angle) {
                existing.push(rel);
            } else {
                grouped.insert(rel.angle, vec![rel]);
            }
        }
    }

    for list in grouped.values_mut() {
        list.sort_by(|a, b| b.magnitude.cmp(&a.magnitude));
    }

    let mut iter_order: Vec<OrderedFloat<f64>> = grouped.keys().copied().collect();
    iter_order.sort();

    let mut ret: Vec<Position> = Vec::new();

    loop {
        let mut found = false;
        for pos in iter_order.iter() {
            let asteroids = grouped.get_mut(pos).unwrap();
            if let Some(next) = asteroids.pop() {
                found = true;
                ret.push(next.pos);
            }
        }
        if !found {
            break;
        }
    }

    ret
}

fn get_day_2_answer(seq: &[Position]) -> i32 {
    let pos = seq.get(200 - 1).unwrap();
    pos.x * 100 + pos.y
}

fn main() -> Result<(), Box<dyn Error>> {
    let map = AsteroidMap::parse(include_str!("input.txt"))?;
    println!("map is width {} height {}", map.width, map.height);
    let detected = get_num_detectable(&map)?;
    println!(
        "detected at ({},{}): {}",
        detected.pos.x, detected.pos.y, detected.num_detectable
    );
    let seq = generate_laser_firing_sequence(&map, &detected);
    println!("day 2 answer: {}", get_day_2_answer(&seq));

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    // Ensure that the angle increases in the right direction.
    // That is, straight up should be the smallest value and angles should increase clockwise.
    #[test]
    fn test_position() {
        let origin = Position::new(2, 2);
        let around = [
            Position::new(2, 1),
            Position::new(3, 1),
            Position::new(3, 2),
            Position::new(3, 3),
            Position::new(2, 3),
            Position::new(1, 3),
            Position::new(1, 2),
            Position::new(1, 1),
        ];

        for (a, b) in around
            .iter()
            .take(around.len() - 1)
            .zip(around.iter().skip(1))
        {
            let rel_a = RelativePosition::new(origin, *a);
            let rel_b = RelativePosition::new(origin, *b);
            assert!(
                rel_a.angle < rel_b.angle,
                "a: {:?} b: {:?} rel_a: {:?} rel_b: {:?}",
                a,
                b,
                rel_a,
                rel_b
            );
        }
    }

    #[test]
    fn test_parse_input() {
        let map = AsteroidMap::parse(
            r#"
.#.
...
"#,
        )
        .unwrap();
        assert_eq!(map.height, 2);
        assert_eq!(map.width, 3);
        assert_eq!(map.asteroids, vec![Position { x: 1, y: 0 }]);
    }

    #[test]
    fn test_case1() {
        let map = AsteroidMap::parse(
            r#"
.#..#
.....
#####
....#
...##
"#,
        )
        .unwrap();

        let result = get_num_detectable(&map).unwrap();
        assert_eq!((result.pos.x, result.pos.y), (3, 4));
        assert_eq!(result.num_detectable, 8);
    }

    #[test]
    fn test_case2() {
        let map = AsteroidMap::parse(
            r#"
......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####
"#,
        )
        .unwrap();

        let result = get_num_detectable(&map).unwrap();
        assert_eq!((result.pos.x, result.pos.y), (5, 8));
        assert_eq!(result.num_detectable, 33);
    }

    #[test]
    fn test_case3() {
        let map = AsteroidMap::parse(
            r#"
#.#...#.#.
.###....#.
.#....#...
##.#.#.#.#
....#.#.#.
.##..###.#
..#...##..
..##....##
......#...
.####.###.
"#,
        )
        .unwrap();

        let result = get_num_detectable(&map).unwrap();
        assert_eq!((result.pos.x, result.pos.y), (1, 2));
        assert_eq!(result.num_detectable, 35);
    }

    #[test]
    fn test_case4() {
        let map = AsteroidMap::parse(
            r#"
.#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#..
"#,
        )
        .unwrap();

        let result = get_num_detectable(&map).unwrap();
        assert_eq!((result.pos.x, result.pos.y), (6, 3));
        assert_eq!(result.num_detectable, 41);
    }

    static BIG_MAP: &'static str = r#"
.#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##
"#;

    #[test]
    fn test_case5() {
        let map = AsteroidMap::parse(BIG_MAP).unwrap();
        let result = get_num_detectable(&map).unwrap();
        assert_eq!((result.pos.x, result.pos.y), (11, 13));
        assert_eq!(result.num_detectable, 210);
    }

    #[test]
    fn test_generate_laser_firing_sequence() {
        let map = AsteroidMap::parse(BIG_MAP).unwrap();
        let place = get_num_detectable(&map).unwrap();
        assert_eq!((place.pos.x, place.pos.y), (11, 13));
        let seq = generate_laser_firing_sequence(&map, &place);

        // offset index by one because the test cases are defined in the terms "first", "second" element,
        // etc.
        assert_eq!(seq.get(1 - 1).unwrap(), &Position::new(11, 12));
        assert_eq!(seq.get(2 - 1).unwrap(), &Position::new(12, 1));
        assert_eq!(seq.get(3 - 1).unwrap(), &Position::new(12, 2));
        assert_eq!(seq.get(10 - 1).unwrap(), &Position::new(12, 8));
        assert_eq!(seq.get(20 - 1).unwrap(), &Position::new(16, 0));
        assert_eq!(seq.get(50 - 1).unwrap(), &Position::new(16, 9));
        assert_eq!(seq.get(100 - 1).unwrap(), &Position::new(10, 16));
        assert_eq!(seq.get(199 - 1).unwrap(), &Position::new(9, 6));
        assert_eq!(seq.get(200 - 1).unwrap(), &Position::new(8, 2));
        assert_eq!(seq.get(201 - 1).unwrap(), &Position::new(10, 9));
        assert_eq!(seq.get(299 - 1).unwrap(), &Position::new(11, 1));

        assert_eq!(802, get_day_2_answer(&seq));
    }
}
