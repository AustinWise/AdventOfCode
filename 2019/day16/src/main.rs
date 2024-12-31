use std::ops::Rem;

fn parse_input(input: &str) -> Vec<i32> {
    input
        .chars()
        .map(|ch| ch.to_string().parse::<i32>().unwrap())
        .collect()
}

// TODO: convert this to a generator that can generate a sequence of arbitrary length
// https://github.com/rust-lang/rust/issues/43122
fn create_pattern(position: usize, min_length: usize) -> Vec<i32> {
    let mut ret: Vec<i32> = Vec::new();
    while ret.len() < min_length {
        ret.append(&mut vec![
            0;
            if ret.is_empty() {
                position
            } else {
                position + 1
            }
        ]);
        ret.append(&mut vec![1; position + 1]);
        ret.append(&mut vec![0; position + 1]);
        ret.append(&mut vec![-1; position + 1]);
        assert!(!ret.is_empty());
    }
    ret
}

fn compute_fft(input: &[i32], cycles: usize) -> Vec<i32> {
    let mut ret = input.to_vec();
    for _ in 1..=cycles {
        let mut new = vec![0; ret.len()];
        for ndx in 0..new.len() {
            let num: i32 = ret
                .iter()
                .zip(create_pattern(ndx, ret.len()))
                .map(|(x, y)| x * y)
                .sum();
            let num = num.abs();
            let num = num.rem(10);
            new[ndx] = num;
        }
        ret = new;
    }
    ret
}

fn main() {
    let nums = parse_input(include_str!("input.txt"));
    let fft = compute_fft(&nums, 100);
    for x in &fft[0..8] {
        print!("{}", x);
    }
    create_pattern(0, 1);
    println!();
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::{compute_fft, create_pattern, parse_input};

    #[test]
    fn test_create_pattern() {
        let pat = create_pattern(0, 4);
        assert_eq!(pat, vec![1, 0, -1, 0, 1, 0, -1]);
        let pat = create_pattern(1, 1);
        assert_eq!(pat, vec![0, 1, 1, 0, 0, -1, -1]);
    }

    #[test]
    fn test_compute_fft() {
        let input = parse_input("12345678");

        let nums = compute_fft(&input, 1);
        assert_eq!(nums, vec![4, 8, 2, 2, 6, 1, 5, 8]);

        let nums = compute_fft(&input, 2);
        assert_eq!(nums, vec![3, 4, 0, 4, 0, 4, 3, 8]);
    }
}
