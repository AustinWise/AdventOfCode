use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::io::BufRead;
use std::io::Write;

#[derive(Debug)]
enum MyError {
    ProgramParseError,
    InvalidParameterMode,
    InvalidOpCode,
    IndexOutOfRange,
    EOF,
    InputParse,
    IoError(std::io::Error),
}

impl Error for MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::ProgramParseError => write!(f, "failed to parse program"),
            MyError::InvalidParameterMode => write!(f, "invalid parameter mode"),
            MyError::InvalidOpCode => write!(f, "invalid opcode"),
            MyError::IndexOutOfRange => write!(f, "index out of range"),
            MyError::EOF => write!(f, "EOF"),
            MyError::InputParse => write!(f, "Input parse"),
            MyError::IoError(io_err) => write!(f, "io error: {}", io_err),
        }
    }
}

fn parse_program(input: &str) -> Result<Vec<i32>, MyError> {
    let mut v: Vec<i32> = Vec::new();
    for num_str in input.split(',') {
        if let Ok(num) = i32::from_str_radix(num_str, 10) {
            v.push(num);
        } else {
            return Err(MyError::ProgramParseError);
        }
    }
    Ok(v)
}

enum ParameterMode {
    Position,
    Immediate,
}

enum Opcode {
    Add(ParameterMode, ParameterMode),
    Multiply(ParameterMode, ParameterMode),
    Input,
    Output(ParameterMode),
    JumpIfTrue(ParameterMode, ParameterMode),
    JumpIfFalse(ParameterMode, ParameterMode),
    LessThan(ParameterMode, ParameterMode),
    Equals(ParameterMode, ParameterMode),
    Exit,
}

fn parse_parameter_mode(parameter_digit: i32) -> Result<ParameterMode, MyError> {
    match parameter_digit {
        0 => Ok(ParameterMode::Position),
        1 => Ok(ParameterMode::Immediate),
        _ => Err(MyError::InvalidParameterMode),
    }
}

fn parse_instruction(instruction: i32) -> Result<Opcode, MyError> {
    let ret = match instruction % 100 {
        1 => Opcode::Add(
            parse_parameter_mode(instruction / 100 % 10)?,
            parse_parameter_mode(instruction / 1000 % 10)?,
        ),
        2 => Opcode::Multiply(
            parse_parameter_mode(instruction / 100 % 10)?,
            parse_parameter_mode(instruction / 1000 % 10)?,
        ),
        3 => Opcode::Input,
        4 => Opcode::Output(parse_parameter_mode(instruction / 100 % 10)?),
        5 => Opcode::JumpIfTrue(
            parse_parameter_mode(instruction / 100 % 10)?,
            parse_parameter_mode(instruction / 1000 % 10)?,
        ),
        6 => Opcode::JumpIfFalse(
            parse_parameter_mode(instruction / 100 % 10)?,
            parse_parameter_mode(instruction / 1000 % 10)?,
        ),
        7 => Opcode::LessThan(
            parse_parameter_mode(instruction / 100 % 10)?,
            parse_parameter_mode(instruction / 1000 % 10)?,
        ),
        8 => Opcode::Equals(
            parse_parameter_mode(instruction / 100 % 10)?,
            parse_parameter_mode(instruction / 1000 % 10)?,
        ),
        99 => Opcode::Exit,
        _ => return Err(MyError::InvalidOpCode),
    };
    let parameter_mode_count = match ret {
        Opcode::Add(_, _) => 2,
        Opcode::Multiply(_, _) => 2,
        Opcode::Input => 0,
        Opcode::Output(_) => 1,
        Opcode::JumpIfTrue(_, _) => 2,
        Opcode::JumpIfFalse(_, _) => 2,
        Opcode::LessThan(_, _) => 2,
        Opcode::Equals(_, _) => 2,
        Opcode::Exit => 0,
    };
    let max_value = (10i32).pow(2 + parameter_mode_count) - 1;
    if instruction > max_value {
        Err(MyError::InvalidOpCode)
    } else {
        Ok(ret)
    }
}

fn get_usize(mem: &[i32], ndx: usize) -> Result<usize, MyError> {
    match usize::try_from(mem[ndx]) {
        Ok(num) => Ok(num),
        Err(_) => Err(MyError::IndexOutOfRange),
    }
}

fn load(mem: &[i32], index: usize, mode: ParameterMode) -> Result<i32, MyError> {
    let ret = match mode {
        ParameterMode::Position => mem[get_usize(mem, index)?],
        ParameterMode::Immediate => mem[index],
    };
    Ok(ret)
}

fn read_number(input: &mut dyn BufRead, output: &mut dyn Write) -> Result<i32, MyError> {
    write!(output, "Please enter a number: ").unwrap();
    output.flush().unwrap();
    let mut buf = String::new();
    match input.read_line(&mut buf) {
        Ok(0) => Err(MyError::EOF),
        Err(io_err) => Err(MyError::IoError(io_err)),
        Ok(_) => match i32::from_str_radix(&buf.trim(), 10) {
            Ok(ret) => Ok(ret),
            Err(_) => Err(MyError::InputParse),
        },
    }
}

fn execute(
    mem: &mut [i32],
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> Result<(), MyError> {
    let mut pc = 0;
    loop {
        match parse_instruction(mem[pc])? {
            Opcode::Add(src1_mode, src2_mode) => {
                let dst = get_usize(&mem, pc + 3)?;
                mem[dst] = load(mem, pc + 1, src1_mode)? + load(mem, pc + 2, src2_mode)?;
                pc += 4;
            }
            Opcode::Multiply(src1_mode, src2_mode) => {
                let dst = get_usize(&mem, pc + 3)?;
                mem[dst] = load(mem, pc + 1, src1_mode)? * load(mem, pc + 2, src2_mode)?;
                pc += 4;
            }
            Opcode::Input => {
                let dst = get_usize(&mem, pc + 1)?;
                mem[dst] = read_number(input, output)?;
                pc += 2;
            }
            Opcode::Output(src_mode) => {
                writeln!(output, "{}", load(mem, pc + 1, src_mode)?).unwrap();
                pc += 2;
            }
            Opcode::JumpIfTrue(comparand_mode, target_mode) => {
                pc = if load(mem, pc + 1, comparand_mode)? != 0 {
                    match usize::try_from(load(mem, pc + 2, target_mode)?) {
                        Ok(loc) => loc,
                        Err(_) => return Err(MyError::IndexOutOfRange),
                    }
                } else {
                    pc + 3
                };
            }
            Opcode::JumpIfFalse(comparand_mode, target_mode) => {
                pc = if load(mem, pc + 1, comparand_mode)? == 0 {
                    match usize::try_from(load(mem, pc + 2, target_mode)?) {
                        Ok(loc) => loc,
                        Err(_) => return Err(MyError::IndexOutOfRange),
                    }
                } else {
                    pc + 3
                };
            }
            Opcode::LessThan(src1_mode, src2_mode) => {
                let dst = get_usize(&mem, pc + 3)?;
                mem[dst] = if load(mem, pc + 1, src1_mode)? < load(mem, pc + 2, src2_mode)? {
                    1
                } else {
                    0
                };
                pc += 4;
            }
            Opcode::Equals(src1_mode, src2_mode) => {
                let dst = get_usize(&mem, pc + 3)?;
                mem[dst] = if load(mem, pc + 1, src1_mode)? == load(mem, pc + 2, src2_mode)? {
                    1
                } else {
                    0
                };
                pc += 4;
            }
            Opcode::Exit => return Ok(()),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = parse_program(&std::fs::read_to_string("input.txt")?)?;
    execute(
        &mut mem,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
    )?;
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
        execute(&mut mem, &mut std::io::empty(), &mut std::io::sink()).expect("execute failed");
        assert_eq!(mem, expected_mem);
    }

    #[test]
    fn test_execute() {
        test_a_program("1,0,0,0,99", "2,0,0,0,99");
        test_a_program("2,3,0,3,99", "2,3,0,6,99");
        test_a_program("2,4,4,5,99,0", "2,4,4,5,99,9801");
        test_a_program("1,1,1,4,99,5,6,0,99", "30,1,1,4,2,5,6,0,99");
        test_a_program("1002,4,3,4,33", "1002,4,3,4,99");
    }

    fn test_io_program(program: &str, input: &str, expected_output: &str) {
        let mut mem = parse_program(program).expect("failed to parse input");
        let mut output = Vec::new();
        execute(&mut mem, &mut std::io::Cursor::new(input), &mut output).expect("execute failed");
        assert_eq!(expected_output, String::from_utf8(output).unwrap());
    }

    #[test]
    fn test_input_output() {
        test_io_program("3,0,4,0,99", "42\n", "Please enter a number: 42\n");
        //equal to 8, position mode
        test_io_program(
            "3,9,8,9,10,9,4,9,99,-1,8",
            "8\n",
            "Please enter a number: 1\n",
        );
        test_io_program(
            "3,9,8,9,10,9,4,9,99,-1,8",
            "42\n",
            "Please enter a number: 0\n",
        );
        //less than 8, position mode
        test_io_program(
            "3,9,7,9,10,9,4,9,99,-1,8",
            "3\n",
            "Please enter a number: 1\n",
        );
        test_io_program(
            "3,9,7,9,10,9,4,9,99,-1,8",
            "8\n",
            "Please enter a number: 0\n",
        );
        test_io_program(
            "3,9,7,9,10,9,4,9,99,-1,8",
            "42\n",
            "Please enter a number: 0\n",
        );
        //equal to 8, immediate mode
        test_io_program(
            "3,3,1108,-1,8,3,4,3,99",
            "8\n",
            "Please enter a number: 1\n",
        );
        test_io_program(
            "3,3,1108,-1,8,3,4,3,99",
            "42\n",
            "Please enter a number: 0\n",
        );
        //less than 8, immediate mode
        test_io_program(
            "3,3,1107,-1,8,3,4,3,99",
            "3\n",
            "Please enter a number: 1\n",
        );
        test_io_program(
            "3,3,1107,-1,8,3,4,3,99",
            "8\n",
            "Please enter a number: 0\n",
        );
        test_io_program(
            "3,3,1107,-1,8,3,4,3,99",
            "42\n",
            "Please enter a number: 0\n",
        );

        //jumps in position mode
        test_io_program(
            "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9",
            "0\n",
            "Please enter a number: 0\n",
        );
        test_io_program(
            "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9",
            "1\n",
            "Please enter a number: 1\n",
        );
        test_io_program(
            "3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9",
            "42\n",
            "Please enter a number: 1\n",
        );
        //jumps in immediate mode
        test_io_program(
            "3,3,1105,-1,9,1101,0,0,12,4,12,99,1",
            "0\n",
            "Please enter a number: 0\n",
        );
        test_io_program(
            "3,3,1105,-1,9,1101,0,0,12,4,12,99,1",
            "1\n",
            "Please enter a number: 1\n",
        );
        test_io_program(
            "3,3,1105,-1,9,1101,0,0,12,4,12,99,1",
            "42\n",
            "Please enter a number: 1\n",
        );

        let around_eight = "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99";
        test_io_program(around_eight, "0\n", "Please enter a number: 999\n");
        test_io_program(around_eight, "1\n", "Please enter a number: 999\n");
        test_io_program(around_eight, "3\n", "Please enter a number: 999\n");
        test_io_program(around_eight, "7\n", "Please enter a number: 999\n");
        test_io_program(around_eight, "8\n", "Please enter a number: 1000\n");
        test_io_program(around_eight, "9\n", "Please enter a number: 1001\n");
        test_io_program(around_eight, "10\n", "Please enter a number: 1001\n");
        test_io_program(around_eight, "42\n", "Please enter a number: 1001\n");
    }
}
