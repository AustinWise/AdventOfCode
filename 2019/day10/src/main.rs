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

fn sort_quadrant<'a, P, F>(asteroids: P, filter: F, flip: bool) -> Vec<(i32, i32)>
where
    P: Iterator<Item = &'a (i32, i32)>,
    F: Fn(i32, i32) -> bool,
{
    let mut ret = Vec::new();
    for &(x, y) in asteroids {
        if filter(x, y) {
            ret.push((x, y));
        }
    }
    ret.sort_by(|&(a_x, a_y), &(b_x, b_y)| {
        if flip {
            Rational32::new(a_x, a_y).cmp(&Rational32::new(b_x, b_y))
        } else {
            Rational32::new(b_x, b_y).cmp(&Rational32::new(a_x, a_y))
        }
    });
    ret
}

fn sort_asteroids_for_laser(placement: &PlacementResult) -> Vec<(i32, i32)> {
    let mut ret: Vec<(i32, i32)> = Vec::new();

    // up
    if placement.asteroids_in_direction.contains_key(&(0, -1)) {
        ret.push((0, -1));
    }

    // top right
    ret.append(&mut sort_quadrant(
        placement.asteroids_in_direction.keys(),
        |x, y| x > 0 && y < 0,
        false,
    ));

    //right
    if placement.asteroids_in_direction.contains_key(&(1, 0)) {
        ret.push((1, 0));
    }

    // bottom right
    ret.append(&mut sort_quadrant(
        placement.asteroids_in_direction.keys(),
        |x, y| x > 0 && y > 0,
        false,
    ));

    // down
    if placement.asteroids_in_direction.contains_key(&(0, -1)) {
        ret.push((0, -1));
    }

    // bottom left
    ret.append(&mut sort_quadrant(
        placement.asteroids_in_direction.keys(),
        |x, y| x < 0 && y < 0,
        true,
    ));

    // left
    if placement.asteroids_in_direction.contains_key(&(-1, 0)) {
        ret.push((-1, 0));
    }

    // top left
    ret.append(&mut sort_quadrant(
        placement.asteroids_in_direction.keys(),
        |x, y| x < 0 && y < 0,
        true,
    ));
    ret
}

fn generate_laser_firing_sequence(place: &PlacementResult) -> Vec<(isize, isize)> {
    let seq = sort_asteroids_for_laser(&place);
    let mut counts = place.asteroids_in_direction.clone();
    let mut ret: Vec<(isize, isize)> = Vec::new();
    let mut cnt = 0;
    loop {
        let mut found = false;
        for &(x, y) in seq.iter() {
            let remaining = *counts.get(&(x, y)).unwrap();
            if remaining > 0 {
                cnt += 1;

                let translated = (place.x as isize + x as isize, place.y as isize + y as isize);

                println!(
                    "asteroid {} at ({}, {}), translated: ({}, {})",
                    cnt, x, y, translated.0, translated.1
                );

                found = true;
                ret.push(translated);
                counts.insert((x, y), remaining - 1);
            }
        }
        if !found {
            break;
        }
    }
    ret
}

fn main() -> Result<(), Box<dyn Error>> {
    let map = AsteroidMap::parse(include_str!("input.txt"))?;
    println!("map is width {} height {}", map.width, map.height);
    let detected = get_num_detectable(&map)?;
    println!(
        "detected at ({},{}): {}",
        detected.x,
        detected.y,
        detected.num_detectable()
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
        assert_eq!((result.x, result.y), (11, 13));
        assert_eq!(result.num_detectable(), 210);
    }

    #[test]
    fn test_sort_quadrant_top_right() {
        // pre-sorted input
        let input = [(1, -3), (2, -2), (3, -1)];
        let result = sort_quadrant(input.iter(), |_, _| true, false);
        assert_eq!(result, input);
    }

    #[test]
    fn test_sort_quadrant_bottom_right() {
        // pre-sorted input
        let input = [(3, 1), (2, 2), (1, 3)];
        let result = sort_quadrant(input.iter(), |_, _| true, false);
        assert_eq!(result, input);
    }

    #[test]
    fn test_sort_quadrant_bottom_left() {
        // pre-sorted input
        let input = [(-1, -3), (-2, -2), (-3, -1)];
        let result = sort_quadrant(input.iter(), |_, _| true, true);
        assert_eq!(result, input);
    }

    #[test]
    fn test_sort_quadrant_top_left() {
        // pre-sorted input
        let input = [(-3, 1), (-2, 2), (-1, 3)];
        let result = sort_quadrant(input.iter(), |_, _| true, true);
        assert_eq!(result, input);
    }

    #[test]
    fn test_generate_laser_firing_sequence() {
        let map = AsteroidMap::parse(BIG_MAP).unwrap();
        let place = get_num_detectable(&map).unwrap();
        assert_eq!((place.x, place.y), (11, 13));
        let seq = generate_laser_firing_sequence(&place);

        // offset index by one because the test cases are defined in the terms "first", "second" element,
        // etc.
        assert_eq!(seq.get(1 - 1).unwrap(), &(11, 12));
        assert_eq!(seq.get(2 - 1).unwrap(), &(12, 1));
        assert_eq!(seq.get(3 - 1).unwrap(), &(12, 2));
        assert_eq!(seq.get(10 - 1).unwrap(), &(12, 8));
        assert_eq!(seq.get(20 - 1).unwrap(), &(16, 0));
        assert_eq!(seq.get(50 - 1).unwrap(), &(16, 9));
        // assert_eq!(seq.get(100 - 1).unwrap(), &(10, 16));
    }
}
