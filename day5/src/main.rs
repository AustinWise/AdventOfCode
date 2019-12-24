use std::convert::TryFrom;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum MyError {
    ProgramParseError,
    InvalidOpCode,
    IndexOutOfRange,
    AnswerNotFound,
}

impl Error for MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::ProgramParseError => write!(f, "failed to parse program"),
            MyError::InvalidOpCode => write!(f, "invalid opcode"),
            MyError::IndexOutOfRange => write!(f, "index out of range"),
            MyError::AnswerNotFound => write!(f, "answer not found"),
        }
    }
}

fn parse_program(input: &str) -> Result<Vec<u32>, MyError> {
    let mut v: Vec<u32> = Vec::new();
    for num_str in input.split(',') {
        if let Ok(num) = u32::from_str_radix(num_str, 10) {
            v.push(num);
        } else {
            return Err(MyError::ProgramParseError);
        }
    }
    Ok(v)
}

fn get_usize(mem: &[u32], ndx: usize) -> Result<usize, MyError> {
    match usize::try_from(mem[ndx]) {
        Ok(num) => Ok(num),
        Err(_) => Err(MyError::IndexOutOfRange),
    }
}

fn execute(mem: &mut [u32]) -> Result<(), MyError> {
    let mut pc = 0;
    loop {
        match mem[pc] {
            1 => {
                let src1 = get_usize(&mem, pc + 1)?;
                let src2 = get_usize(&mem, pc + 2)?;
                let dst = get_usize(&mem, pc + 3)?;
                mem[dst] = mem[src1] + mem[src2];
                pc += 4;
            }
            2 => {
                let src1 = get_usize(&mem, pc + 1)?;
                let src2 = get_usize(&mem, pc + 2)?;
                let dst = get_usize(&mem, pc + 3)?;
                mem[dst] = mem[src1] * mem[src2];
                pc += 4;
            }
            99 => return Ok(()),
            _ => return Err(MyError::InvalidOpCode),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mem = parse_program(&std::fs::read_to_string("input.txt")?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        parse_program("").expect_err("parse failed to fail");
        parse_program("turtle").expect_err("parse failed to fail");
        assert_eq!(vec![0], parse_program("0").expect("parse failed"));
        assert_eq!(vec![1, 2], parse_program("1,2").expect("parse failed"));
    }

    fn test_a_program(input: &str, expected_output: &str) {
        let mut mem = parse_program(input).expect("failed to parse input");
        let expected_mem = parse_program(expected_output).expect("failed to parse input");
        execute(&mut mem).expect("execute failed");
        assert_eq!(mem, expected_mem);
    }

    #[test]
    fn test_execute() {
        test_a_program("1,0,0,0,99", "2,0,0,0,99");
        test_a_program("2,3,0,3,99", "2,3,0,6,99");
        test_a_program("2,4,4,5,99,0", "2,4,4,5,99,9801");
        test_a_program("1,1,1,4,99,5,6,0,99", "30,1,1,4,2,5,6,0,99");
    }
}
