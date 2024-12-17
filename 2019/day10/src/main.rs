use num_rational::Rational32;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::ops::Neg;
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

struct AsteroidMap {
    width: usize,
    height: usize,
    // true if occupied by an asteroid.
    // contains height-number elements, each of which contains width-number bools.
    starfield: Vec<Vec<bool>>,
}

impl AsteroidMap {
    fn parse(input: &str) -> Result<AsteroidMap, ParseError> {
        let mut map: Vec<Vec<bool>> = Vec::new();
        for line in input.split('\n') {
            if !line.is_empty() {
                let mut map_line: Vec<bool> = Vec::new();
                for c in line.chars() {
                    map_line.push(match c {
                        '.' => false,
                        '#' => true,
                        _ => return Err(ParseError::UnexpectedCharacter(c)),
                    });
                }
                if let Some(first_line) = map.first() {
                    if first_line.len() != map_line.len() {
                        return Err(ParseError::UnevenLineLength);
                    }
                }
                map.push(map_line);
            }
        }

        if map.is_empty() {
            return Err(ParseError::NoLines);
        }

        Ok(AsteroidMap {
            width: map.first().unwrap().len(),
            height: map.len(),
            starfield: map,
        })
    }
}

struct PlacementResult {
    x: usize,
    y: usize,
    asteroids_in_direction: HashMap<(i32, i32), usize>,
}

impl PlacementResult {
    fn num_detectable(&self) -> usize {
        self.asteroids_in_direction.len()
    }
}

#[derive(Error, Debug)]
enum PlacementError {
    #[error("no asteroids in starfield")]
    NoAsteroids,
}

fn get_delta(x_origin: usize, y_origin: usize, x: usize, y: usize) -> Option<(i32, i32)> {
    match (x_origin.cmp(&x), y_origin.cmp(&y)) {
        (Ordering::Equal, Ordering::Equal) => None,
        // special case axis-aligned locations, to avoid having to deal with
        // divide by zero
        (Ordering::Equal, Ordering::Greater) => Some((0, 1)),
        (Ordering::Equal, Ordering::Less) => Some((0, -1)),
        (Ordering::Less, Ordering::Equal) => Some((-1, 0)),
        (Ordering::Greater, Ordering::Equal) => Some((1, 0)),
        // normal cases
        (_, _) => {
            // TODO: maybe check for overflow
            let delta_x = x as i32 - x_origin as i32;
            let delta_y = y as i32 - y_origin as i32;
            // use a rational number to reduce the fraction
            let rat = Rational32::new(delta_x, delta_y);
            let delta_x = if delta_x < 0 {
                rat.numer().abs().neg()
            } else {
                rat.numer().abs()
            };
            let delta_y = if delta_y < 0 {
                rat.denom().abs().neg()
            } else {
                rat.denom().abs()
            };
            Some((delta_x, delta_y))
        }
    }
}

fn count_asteroids_in_line_of_sight(
    map: &AsteroidMap,
    x_origin: usize,
    y_origin: usize,
) -> PlacementResult {
    let mut asteroids: HashMap<(i32, i32), usize> = HashMap::new();
    for (y, line) in map.starfield.iter().enumerate() {
        for (x, &has_asteroid) in line.iter().enumerate() {
            if has_asteroid {
                if let Some((delta_x, delta_y)) = get_delta(x_origin, y_origin, x, y) {
                    if let Some(count) = asteroids.get(&(delta_x, delta_y)) {
                        asteroids.insert((delta_x, delta_y), count + 1);
                    } else {
                        asteroids.insert((delta_x, delta_y), 1);
                    }
                }
            }
        }
    }
    PlacementResult {
        x: x_origin,
        y: y_origin,
        asteroids_in_direction: asteroids,
    }
}

fn get_num_detectable(map: &AsteroidMap) -> Result<PlacementResult, PlacementError> {
    let mut ret: Option<PlacementResult> = None;
    for (y, line) in map.starfield.iter().enumerate() {
        for (x, &has_asteroid) in line.iter().enumerate() {
            if has_asteroid {
                let candidate = count_asteroids_in_line_of_sight(map, x, y);
                if ret.is_none()
                    || ret.as_ref().unwrap().num_detectable() < candidate.num_detectable()
                {
                    ret = Some(candidate);
                }
            }
        }
    }
    if let Some(ret) = ret {
        Ok(ret)
    } else {
        Err(PlacementError::NoAsteroids)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let map = AsteroidMap::parse(include_str!("input.txt"))?;
    println!("map is width {} height {}", map.width, map.height);
    let detected = get_num_detectable(&map)?;
    println!(
        "detected at ({},{}): {}",
        detected.x, detected.y, detected.num_detectable()
    );
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

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
        assert_eq!(
            map.starfield,
            vec![vec![false, true, false], vec![false, false, false]]
        );
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
        assert_eq!((result.x, result.y), (3, 4));
        assert_eq!(result.num_detectable(), 8);
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
        assert_eq!((result.x, result.y), (5, 8));
        assert_eq!(result.num_detectable(), 33);
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
        assert_eq!((result.x, result.y), (1, 2));
        assert_eq!(result.num_detectable(), 35);
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
        assert_eq!((result.x, result.y), (6, 3));
        assert_eq!(result.num_detectable(), 41);
    }

    #[test]
    fn test_case5() {
        let map = AsteroidMap::parse(
            r#"
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
"#,
        )
        .unwrap();

        let result = get_num_detectable(&map).unwrap();
        assert_eq!((result.x, result.y), (11, 13));
        assert_eq!(result.num_detectable(), 210);
    }
}
