use std::convert::TryInto;
use std::error::Error;

extern crate intcode;

//TODO: make this not allocate so much?
//TODO: make this a generator?
fn permutation(number_of_perms: u32) -> Vec<Vec<i32>> {
    fn inner(result: &mut Vec<Vec<i32>>, prefix: &Vec<i32>, to_perm: &Vec<i32>) {
        if to_perm.len() == 0 {
            result.push(prefix.to_owned());
        } else {
            for i in 0..to_perm.len() {
                let mut prefix = prefix.to_owned();
                prefix.push(to_perm[i]);
                let mut to_perm = to_perm.to_owned();
                to_perm.remove(i);
                inner(result, &prefix, &to_perm);
            }
        }
    }

    let mut to_perm: Vec<i32> = Vec::new();
    for i in 0..number_of_perms {
        to_perm.push(i.try_into().unwrap());
    }

    let mut ret = Vec::new();
    if number_of_perms != 0 {
        inner(&mut ret, &Vec::new(), &to_perm);
    }
    ret
}

fn main() -> Result<(), Box<dyn Error>> {
    //let mut mem = intcode::parse_program(&std::fs::read_to_string("input.txt")?)?;

    let things = permutation(5);
    for f in things {
        println!("{:?}", f);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_perm() {
        let empty: Vec<Vec<i32>> = Vec::new();
        assert_eq!(permutation(0), empty);
        assert_eq!(permutation(1), [[0]]);
        assert_eq!(permutation(2), [[0, 1], [1, 0]]);
        assert_eq!(permutation(3), [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0]]);
    }
}
