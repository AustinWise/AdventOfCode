use std::convert::TryInto;
use std::error::Error;

extern crate intcode;

//TODO: make this not allocate so much?
//TODO: make this a generator?
fn permutation(begin: i32, number_of_perms: u32) -> Vec<Vec<i32>> {
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
        let signed: i32 = i.try_into().unwrap();
        to_perm.push(begin + signed);
    }

    let mut ret = Vec::new();
    if number_of_perms != 0 {
        inner(&mut ret, &Vec::new(), &to_perm);
    }
    ret
}

fn run_amplifier_controller_program(
    program: &Vec<i32>,
    phase_setting: &Vec<i32>,
) -> Result<i32, Box<dyn Error>> {
    let mut input: i32 = 0;
    for phase in phase_setting {
        let input_str = format!("{}\n{}\n", phase, input);
        let mut output_buf = Vec::new();
        let mut program_copy = program.to_owned();
        intcode::execute_no_prompt(
            &mut program_copy,
            &mut std::io::Cursor::new(input_str),
            &mut output_buf,
        )?;
        let output_str = std::string::String::from_utf8(output_buf)?;
        input = output_str.trim().parse()?;
    }
    Ok(input)
}

fn find_max_thruster(program: &Vec<i32>) -> Result<i32, Box<dyn Error>> {
    let mut max_thrust = i32::min_value();

    for phase_setting in permutation(0, 5) {
        max_thrust = max_thrust.max(run_amplifier_controller_program(program, &phase_setting)?);
    }

    Ok(max_thrust)
}

fn main() -> Result<(), Box<dyn Error>> {
    let program = intcode::parse_program(&std::fs::read_to_string("input.txt")?)?;

    println!("max thrust: {}", find_max_thruster(&program)?);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_perm() {
        let empty: Vec<Vec<i32>> = Vec::new();
        assert_eq!(permutation(0, 0), empty);
        assert_eq!(permutation(0, 1), [[0]]);
        assert_eq!(permutation(0, 2), [[0, 1], [1, 0]]);
        assert_eq!(
            permutation(0, 3),
            [
                [0, 1, 2],
                [0, 2, 1],
                [1, 0, 2],
                [1, 2, 0],
                [2, 0, 1],
                [2, 1, 0]
            ]
        );
    }

    #[test]
    fn test_perm_oft() {
        let empty: Vec<Vec<i32>> = Vec::new();
        assert_eq!(permutation(5, 0), empty);
        assert_eq!(permutation(5, 1), [[5]]);
        assert_eq!(permutation(5, 2), [[5, 6], [6, 5]]);
        assert_eq!(
            permutation(5, 3),
            [
                [5, 6, 7],
                [5, 7, 6],
                [6, 5, 7],
                [6, 7, 5],
                [7, 5, 6],
                [7, 6, 5]
            ]
        );
    }

    #[test]
    fn test_find_max_thrust() {
        let program = intcode::parse_program("3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0")
            .expect("parse program");
        assert_eq!(43210, find_max_thruster(&program).expect("find max thrust"));

        let program = intcode::parse_program(
            "3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0",
        )
        .expect("parse program");
        assert_eq!(54321, find_max_thruster(&program).expect("find max thrust"));

        let program = intcode::parse_program("3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0").expect("parse program");
        assert_eq!(65210, find_max_thruster(&program).expect("find max thrust"));
    }
}
