// NOTE: this is terribly slow, part 2 takes a couple of hours.

use std::ops::Rem;

fn parse_input(input: &str) -> Vec<i32> {
    input
        .chars()
        .map(|ch| ch.to_string().parse::<i32>().unwrap())
        .collect()
}

struct PatternIterator {
    ndx: usize,
    length: usize,
}

impl Iterator for PatternIterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = match self.ndx / self.length {
            0 => 0,
            1 => 1,
            2 => 0,
            3 => -1,
            4 => {
                self.ndx = 1;
                return Some(0);
            }
            _ => panic!("should not be possible"),
        };

        self.ndx += 1;

        Some(ret)
    }
}

impl PatternIterator {
    fn new(position: usize, offset: usize) -> Self {
        Self {
            ndx: offset + 1,
            length: position + 1,
        }
    }
}

fn compute_fft(input: &[i32], offset: usize, cycles: usize) -> Vec<i32> {
    let mut ret = input.to_vec();

    for x in 1..=cycles {
        let mut new = vec![0; ret.len()];

        for ndx in offset..new.len() {
            let num: i32 = ret
                .iter()
                .skip(offset)
                .zip(PatternIterator::new(ndx, offset))
                .map(|(x, y)| x * y)
                .sum();
            let num = num.abs();
            let num = num.rem(10);

            if ndx % 1000 == 0 {
                println!("iter: {} {}/{}", x, ndx, new.len());
            }
            new[ndx] = num;
        }
        println!("iter: {}", x);
        ret = new;
    }
    ret
}

fn part_1(input: &str) -> String {
    let nums = parse_input(input);
    let fft = compute_fft(&nums, 0, 100);
    fft.iter().take(8).map(|i| i.to_string()).collect()
}

fn part_2(input: &str) -> String {
    let msg_offset = input[0..7].parse::<usize>().unwrap();
    println!("msg_offset: {}", msg_offset);
    let nums = parse_input(&input.repeat(10000));
    assert!(
        msg_offset < nums.len(),
        "msg_offset: {} nums.len(): {}",
        msg_offset,
        nums.len()
    );
    let fft = compute_fft(&nums, msg_offset, 100);
    assert!(msg_offset < fft.len());
    fft.iter()
        .skip(msg_offset)
        .take(8)
        .map(|i| i.to_string())
        .collect()
}

fn main() {
    let input = include_str!("input.txt");
    let p1 = part_1(input);
    println!("part 1: {}", p1);
    println!("part 2: {}", part_2(input));
    println!("part 1 (repeated): {}", p1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pattern() {
        let pat: Vec<i32> = PatternIterator::new(0, 0).take(7).collect();
        assert_eq!(pat, vec![1, 0, -1, 0, 1, 0, -1]);
        let pat: Vec<i32> = PatternIterator::new(1, 0).take(7).collect();
        assert_eq!(pat, vec![0, 1, 1, 0, 0, -1, -1]);
        let pat: Vec<i32> = PatternIterator::new(0, 2).take(5).collect();
        assert_eq!(pat, vec![-1, 0, 1, 0, -1]);
    }

    #[test]
    fn test_compute_fft() {
        let input = parse_input("12345678");

        let nums = compute_fft(&input, 0, 1);
        assert_eq!(nums, vec![4, 8, 2, 2, 6, 1, 5, 8]);

        let nums = compute_fft(&input, 0, 2);
        assert_eq!(nums, vec![3, 4, 0, 4, 0, 4, 3, 8]);
    }

    #[test]
    fn test_part2() {
        // assert_eq!("84462026", part_2("03036732577212944063491565474664"));
    }
}
