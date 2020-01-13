use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::io::BufRead;
use std::io::Write;
use std::num::ParseIntError;
use std::sync::mpsc::{Receiver, RecvError, SendError, SyncSender};

#[derive(Debug)]
pub enum IntcodeError {
    ProgramParseError,
    InvalidParameterMode,
    InvalidOpCode,
    IndexOutOfRange,
    EOF,
    IntParse(ParseIntError),
    IoError(std::io::Error),
    RecvError(RecvError),
    SendError(SendError<i32>),
}

impl Error for IntcodeError {}

impl From<ParseIntError> for IntcodeError {
    fn from(err: ParseIntError) -> IntcodeError {
        IntcodeError::IntParse(err)
    }
}

impl From<std::io::Error> for IntcodeError {
    fn from(err: std::io::Error) -> IntcodeError {
        IntcodeError::IoError(err)
    }
}

impl From<RecvError> for IntcodeError {
    fn from(err: RecvError) -> IntcodeError {
        IntcodeError::RecvError(err)
    }
}

impl From<SendError<i32>> for IntcodeError {
    fn from(err: SendError<i32>) -> IntcodeError {
        IntcodeError::SendError(err)
    }
}

impl fmt::Display for IntcodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntcodeError::ProgramParseError => write!(f, "failed to parse program"),
            IntcodeError::InvalidParameterMode => write!(f, "invalid parameter mode"),
            IntcodeError::InvalidOpCode => write!(f, "invalid opcode"),
            IntcodeError::IndexOutOfRange => write!(f, "index out of range"),
            IntcodeError::EOF => write!(f, "EOF"),
            IntcodeError::IntParse(int_parse_error) => write!(f, "Int parse: {}", int_parse_error),
            IntcodeError::IoError(io_err) => write!(f, "io error: {}", io_err),
            IntcodeError::RecvError(recv_err) => write!(f, "recv error: {}", recv_err),
            IntcodeError::SendError(send_err) => write!(f, "send error: {}", send_err),
        }
    }
}

pub fn parse_program(input: &str) -> Result<Vec<i32>, IntcodeError> {
    let mut v: Vec<i32> = Vec::new();
    for num_str in input.trim().split(',') {
        if let Ok(num) = i32::from_str_radix(num_str, 10) {
            v.push(num);
        } else {
            return Err(IntcodeError::ProgramParseError);
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

fn parse_parameter_mode(parameter_digit: i32) -> Result<ParameterMode, IntcodeError> {
    match parameter_digit {
        0 => Ok(ParameterMode::Position),
        1 => Ok(ParameterMode::Immediate),
        _ => Err(IntcodeError::InvalidParameterMode),
    }
}

fn parse_instruction(instruction: i32) -> Result<Opcode, IntcodeError> {
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
        _ => return Err(IntcodeError::InvalidOpCode),
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
        Err(IntcodeError::InvalidOpCode)
    } else {
        Ok(ret)
    }
}

fn get_usize(mem: &[i32], ndx: usize) -> Result<usize, IntcodeError> {
    match usize::try_from(mem[ndx]) {
        Ok(num) => Ok(num),
        Err(_) => Err(IntcodeError::IndexOutOfRange),
    }
}

fn load(mem: &[i32], index: usize, mode: ParameterMode) -> Result<i32, IntcodeError> {
    let ret = match mode {
        ParameterMode::Position => mem[get_usize(mem, index)?],
        ParameterMode::Immediate => mem[index],
    };
    Ok(ret)
}

trait ReadNumber {
    fn read_number(&mut self) -> Result<i32, IntcodeError>;
}

struct BufReadNumber<'a> {
    buf_read: &'a mut dyn BufRead,
}

struct ChannelReadNumber<'a> {
    input: &'a Receiver<i32>,
}

impl ReadNumber for BufReadNumber<'_> {
    fn read_number(&mut self) -> Result<i32, IntcodeError> {
        let mut buf = String::new();
        match self.buf_read.read_line(&mut buf)? {
            0 => Err(IntcodeError::EOF),
            _ => Ok(i32::from_str_radix(&buf.trim(), 10)?),
        }
    }
}

impl ReadNumber for ChannelReadNumber<'_> {
    fn read_number(&mut self) -> Result<i32, IntcodeError> {
        Ok(self.input.recv()?)
    }
}

trait WriteNumber {
    fn write_number(&mut self, num: i32) -> Result<(), IntcodeError>;
    fn prompt_for_number(&mut self) -> Result<(), IntcodeError>;
}

struct WriteWriteNumber<'a> {
    output: &'a mut dyn Write,
    prompt: bool,
}

struct ChannelWriteNumber {
    output: SyncSender<i32>,
}

impl WriteNumber for WriteWriteNumber<'_> {
    fn write_number(&mut self, num: i32) -> Result<(), IntcodeError> {
        writeln!(self.output, "{}", num)?;
        Ok(())
    }

    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        if self.prompt {
            write!(self.output, "Please enter a number: ")?;
            self.output.flush()?;
        }
        Ok(())
    }
}

impl WriteNumber for ChannelWriteNumber {
    fn write_number(&mut self, num: i32) -> Result<(), IntcodeError> {
        self.output.send(num)?;
        Ok(())
    }

    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        Ok(())
    }
}

fn execute_inner(
    mem: &mut [i32],
    input: &mut dyn ReadNumber,
    output: &mut dyn WriteNumber,
) -> Result<(), IntcodeError> {
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
                output.prompt_for_number()?;
                let dst = get_usize(&mem, pc + 1)?;
                mem[dst] = input.read_number()?;
                pc += 2;
            }
            Opcode::Output(src_mode) => {
                output.write_number(load(mem, pc + 1, src_mode)?)?;
                pc += 2;
            }
            Opcode::JumpIfTrue(comparand_mode, target_mode) => {
                pc = if load(mem, pc + 1, comparand_mode)? != 0 {
                    match usize::try_from(load(mem, pc + 2, target_mode)?) {
                        Ok(loc) => loc,
                        Err(_) => return Err(IntcodeError::IndexOutOfRange),
                    }
                } else {
                    pc + 3
                };
            }
            Opcode::JumpIfFalse(comparand_mode, target_mode) => {
                pc = if load(mem, pc + 1, comparand_mode)? == 0 {
                    match usize::try_from(load(mem, pc + 2, target_mode)?) {
                        Ok(loc) => loc,
                        Err(_) => return Err(IntcodeError::IndexOutOfRange),
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

pub fn execute(
    mem: &mut [i32],
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> Result<(), IntcodeError> {
    let mut input_trait_object = BufReadNumber { buf_read: input };
    let mut output_trait_object = WriteWriteNumber {
        output: output,
        prompt: true,
    };
    execute_inner(mem, &mut input_trait_object, &mut output_trait_object)
}

pub fn execute_no_prompt(
    mem: &mut [i32],
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> Result<(), IntcodeError> {
    let mut input_trait_object = BufReadNumber { buf_read: input };
    let mut output_trait_object = WriteWriteNumber {
        output: output,
        prompt: false,
    };
    execute_inner(mem, &mut input_trait_object, &mut output_trait_object)
}

pub fn execute_no_io(mem: &mut [i32]) -> Result<(), IntcodeError> {
    execute(mem, &mut std::io::empty(), &mut std::io::sink())
}

pub fn execute_with_channel(
    mem: &mut [i32],
    input: &Receiver<i32>,
    output: SyncSender<i32>,
) -> Result<(), IntcodeError> {
    let mut input_trait_object = ChannelReadNumber { input: &input };
    let mut output_trait_object = ChannelWriteNumber { output: output };
    execute_inner(mem, &mut input_trait_object, &mut output_trait_object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::sync_channel;

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
        execute_no_io(&mut mem).expect("execute failed");
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

    fn test_channel_io_helper(input: i32, expected_output: i32) {
        let around_eight = "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99";
        let mut mem = parse_program(around_eight).expect("failed to parse input");
        let (input_send, input_recv) = sync_channel(1);
        let (output_send, output_recv) = sync_channel(1);
        input_send.send(input).expect("failed to send input");
        execute_with_channel(&mut mem, &input_recv, output_send).expect("failed to execute");
        assert_eq!(expected_output, output_recv.recv().expect("failed to recv"));
    }

    #[test]
    fn test_channel_io() {
        test_channel_io_helper(0, 999);
        test_channel_io_helper(1, 999);
        test_channel_io_helper(3, 999);
        test_channel_io_helper(7, 999);
        test_channel_io_helper(8, 1000);
        test_channel_io_helper(9, 1001);
        test_channel_io_helper(10, 1001);
        test_channel_io_helper(42, 1001);
    }
}
