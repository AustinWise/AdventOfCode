use std::collections::HashMap;
use std::convert::TryInto;
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
    InvalidOpCode(i64),
    IndexOutOfRange,
    EOF,
    IntParse(ParseIntError),
    IoError(std::io::Error),
    RecvError(RecvError),
    SendError(SendError<i64>),
    UserInitiatedExit,
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

impl From<SendError<i64>> for IntcodeError {
    fn from(err: SendError<i64>) -> IntcodeError {
        IntcodeError::SendError(err)
    }
}

impl fmt::Display for IntcodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IntcodeError::ProgramParseError => write!(f, "failed to parse program"),
            IntcodeError::InvalidParameterMode => write!(f, "invalid parameter mode"),
            IntcodeError::InvalidOpCode(invalid) => write!(f, "invalid opcode: {}", invalid),
            IntcodeError::IndexOutOfRange => write!(f, "index out of range"),
            IntcodeError::EOF => write!(f, "EOF"),
            IntcodeError::IntParse(int_parse_error) => write!(f, "Int parse: {}", int_parse_error),
            IntcodeError::IoError(io_err) => write!(f, "io error: {}", io_err),
            IntcodeError::RecvError(recv_err) => write!(f, "recv error: {}", recv_err),
            IntcodeError::SendError(send_err) => write!(f, "send error: {}", send_err),
            IntcodeError::UserInitiatedExit => write!(f, "user initiated exit"),
        }
    }
}

pub fn parse_program(input: &str) -> Result<Vec<i64>, IntcodeError> {
    let mut v: Vec<i64> = Vec::new();
    for num_str in input.trim().split(',') {
        if let Ok(num) = num_str.parse::<i64>() {
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
    Relative,
}

enum Opcode {
    Add(ParameterMode, ParameterMode, ParameterMode),
    Multiply(ParameterMode, ParameterMode, ParameterMode),
    Input(ParameterMode),
    Output(ParameterMode),
    JumpIfTrue(ParameterMode, ParameterMode),
    JumpIfFalse(ParameterMode, ParameterMode),
    LessThan(ParameterMode, ParameterMode, ParameterMode),
    Equals(ParameterMode, ParameterMode, ParameterMode),
    AdjustsRelativeBase(ParameterMode),
    Exit,
}

fn parse_parameter_mode(parameter_digit: i64) -> Result<ParameterMode, IntcodeError> {
    match parameter_digit {
        0 => Ok(ParameterMode::Position),
        1 => Ok(ParameterMode::Immediate),
        2 => Ok(ParameterMode::Relative),
        _ => Err(IntcodeError::InvalidParameterMode),
    }
}

fn parse_instruction(instruction: i64) -> Result<Opcode, IntcodeError> {
    let ret = match instruction % 100 {
        1 => Opcode::Add(
            parse_parameter_mode(instruction / 100 % 10)?,
            parse_parameter_mode(instruction / 1000 % 10)?,
            parse_parameter_mode(instruction / 10000 % 10)?,
        ),
        2 => Opcode::Multiply(
            parse_parameter_mode(instruction / 100 % 10)?,
            parse_parameter_mode(instruction / 1000 % 10)?,
            parse_parameter_mode(instruction / 10000 % 10)?,
        ),
        3 => Opcode::Input(parse_parameter_mode(instruction / 100 % 10)?),
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
            parse_parameter_mode(instruction / 10000 % 10)?,
        ),
        8 => Opcode::Equals(
            parse_parameter_mode(instruction / 100 % 10)?,
            parse_parameter_mode(instruction / 1000 % 10)?,
            parse_parameter_mode(instruction / 10000 % 10)?,
        ),
        9 => Opcode::AdjustsRelativeBase(parse_parameter_mode(instruction / 100 % 10)?),
        99 => Opcode::Exit,
        invalid => return Err(IntcodeError::InvalidOpCode(invalid)),
    };
    let parameter_mode_count = match ret {
        Opcode::Add(_, _, _) => 3,
        Opcode::Multiply(_, _, _) => 3,
        Opcode::Input(_) => 1,
        Opcode::Output(_) => 1,
        Opcode::JumpIfTrue(_, _) => 2,
        Opcode::JumpIfFalse(_, _) => 2,
        Opcode::LessThan(_, _, _) => 3,
        Opcode::Equals(_, _, _) => 3,
        Opcode::AdjustsRelativeBase(_) => 1,
        Opcode::Exit => 0,
    };
    let max_value = (10i64).pow(2 + parameter_mode_count) - 1;
    if instruction > max_value {
        Err(IntcodeError::InvalidOpCode(instruction))
    } else {
        Ok(ret)
    }
}

trait ReadNumber {
    fn read_number(&mut self) -> Result<i64, IntcodeError>;
}

struct BufReadNumber<'a> {
    buf_read: &'a mut dyn BufRead,
}

struct ChannelReadNumber<'a> {
    input: &'a Receiver<i64>,
}

impl ReadNumber for BufReadNumber<'_> {
    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        let mut buf = String::new();
        match self.buf_read.read_line(&mut buf)? {
            0 => Err(IntcodeError::EOF),
            _ => Ok((buf.trim()).parse::<i64>()?),
        }
    }
}

impl ReadNumber for ChannelReadNumber<'_> {
    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        Ok(self.input.recv()?)
    }
}

trait WriteNumber {
    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError>;
    fn prompt_for_number(&mut self) -> Result<(), IntcodeError>;
}

struct WriteWriteNumber<'a> {
    output: &'a mut dyn Write,
    prompt: bool,
}

struct ChannelWriteNumber {
    output: SyncSender<i64>,
}

impl WriteNumber for WriteWriteNumber<'_> {
    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
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
    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
        self.output.send(num)?;
        Ok(())
    }

    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        Ok(())
    }
}

pub trait CpuIo {
    fn read_number(&mut self) -> Result<i64, IntcodeError>;
    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError>;
    fn prompt_for_number(&mut self) -> Result<(), IntcodeError>;
}

pub trait AsciiCpuIo {
    fn get_input_line_for_program(&mut self) -> Result<String, IntcodeError>;
    fn accept_output_line_from_program(&mut self, output: &str) -> Result<(), IntcodeError>;
}

struct ComposedCpuIo<'a, R, W>
where
    R: ReadNumber,
    W: WriteNumber,
{
    reader: &'a mut R,
    writer: &'a mut W,
}

impl<R, W> CpuIo for ComposedCpuIo<'_, R, W>
where
    R: ReadNumber,
    W: WriteNumber,
{
    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        self.reader.read_number()
    }
    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
        self.writer.write_number(num)
    }
    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        self.writer.prompt_for_number()
    }
}

struct CpuState<'a, IO>
where
    IO: CpuIo,
{
    pc: i64,
    relative_base: i64,
    mem: HashMap<i64, i64>,
    io: &'a mut IO,
}

impl<IO> CpuState<'_, IO>
where
    IO: CpuIo,
{
    fn create<'a>(mem: &[i64], io: &'a mut IO) -> CpuState<'a, IO> {
        let mut mem_map: HashMap<i64, i64> = HashMap::with_capacity(mem.len()); //TODO: consider using a faster hasher
        for (i, num) in mem.iter().enumerate() {
            mem_map.insert(i.try_into().unwrap(), *num);
        }
        CpuState {
            pc: 0,
            relative_base: 0,
            mem: mem_map,
            io,
        }
    }

    fn load_raw(&self, index: i64) -> Result<i64, IntcodeError> {
        if index < 0 {
            Err(IntcodeError::IndexOutOfRange)
        } else if let Some(num) = self.mem.get(&index) {
            Ok(*num)
        } else {
            Ok(0)
        }
    }

    fn store_raw(&mut self, index: i64, value: i64) -> Result<(), IntcodeError> {
        if index < 0 {
            Err(IntcodeError::IndexOutOfRange)
        } else {
            self.mem.insert(index, value);
            Ok(())
        }
    }

    fn load_effective_address(
        &self,
        pc_rel: i64,
        mode: ParameterMode,
    ) -> Result<i64, IntcodeError> {
        let ret = match mode {
            ParameterMode::Position => self.load_raw(self.pc + pc_rel)?,
            ParameterMode::Immediate => self.pc + pc_rel,
            ParameterMode::Relative => self.relative_base + self.load_raw(self.pc + pc_rel)?,
        };
        Ok(ret)
    }

    fn load(&self, pc_rel: i64, mode: ParameterMode) -> Result<i64, IntcodeError> {
        self.load_raw(self.load_effective_address(pc_rel, mode)?)
    }

    fn store(&mut self, pc_rel: i64, mode: ParameterMode, value: i64) -> Result<(), IntcodeError> {
        self.store_raw(self.load_effective_address(pc_rel, mode)?, value)
    }

    fn execute_inner(&mut self) -> Result<(), IntcodeError> {
        loop {
            match parse_instruction(self.load_raw(self.pc)?)? {
                Opcode::Add(src1_mode, src2_mode, dst_mode) => {
                    self.store(
                        3,
                        dst_mode,
                        self.load(1, src1_mode)? + self.load(2, src2_mode)?,
                    )?;
                    self.pc += 4;
                }
                Opcode::Multiply(src1_mode, src2_mode, dst_mode) => {
                    self.store(
                        3,
                        dst_mode,
                        self.load(1, src1_mode)? * self.load(2, src2_mode)?,
                    )?;
                    self.pc += 4;
                }
                Opcode::Input(dst_mode) => {
                    self.io.prompt_for_number()?;
                    let value = self.io.read_number()?;
                    self.store(1, dst_mode, value)?;
                    self.pc += 2;
                }
                Opcode::Output(src_mode) => {
                    self.io.write_number(self.load(1, src_mode)?)?;
                    self.pc += 2;
                }
                Opcode::JumpIfTrue(comparand_mode, target_mode) => {
                    self.pc = if self.load(1, comparand_mode)? != 0 {
                        self.load(2, target_mode)?
                    } else {
                        self.pc + 3
                    };
                }
                Opcode::JumpIfFalse(comparand_mode, target_mode) => {
                    self.pc = if self.load(1, comparand_mode)? == 0 {
                        self.load(2, target_mode)?
                    } else {
                        self.pc + 3
                    };
                }
                Opcode::LessThan(src1_mode, src2_mode, dst_mode) => {
                    self.store(
                        3,
                        dst_mode,
                        if self.load(1, src1_mode)? < self.load(2, src2_mode)? {
                            1
                        } else {
                            0
                        },
                    )?;
                    self.pc += 4;
                }
                Opcode::Equals(src1_mode, src2_mode, dst_mode) => {
                    self.store(
                        3,
                        dst_mode,
                        if self.load(1, src1_mode)? == self.load(2, src2_mode)? {
                            1
                        } else {
                            0
                        },
                    )?;
                    self.pc += 4;
                }
                Opcode::AdjustsRelativeBase(mode) => {
                    self.relative_base += self.load(1, mode)?;
                    self.pc += 2;
                }
                Opcode::Exit => return Ok(()),
            }
        }
    }

    fn execute(&mut self) -> Result<(), IntcodeError> {
        let rc = self.execute_inner();
        match rc {
            Err(IntcodeError::UserInitiatedExit) => Ok(()),
            _ => rc,
        }
    }
}

pub fn execute_with_io<IO>(mem: &mut [i64], io: &mut IO) -> Result<(), IntcodeError>
where
    IO: CpuIo,
{
    let mut cpu = CpuState::create(mem, io);
    if let Err(err) = cpu.execute() {
        Err(err)
    } else {
        //Copy the changed memory back into the input array
        for (i, value) in mem.iter_mut().enumerate() {
            *value = cpu.load_raw(i.try_into().unwrap())?;
        }
        Ok(())
    }
}

fn execute_composed<R, W>(
    mem: &mut [i64],
    input: &mut R,
    output: &mut W,
) -> Result<(), IntcodeError>
where
    R: ReadNumber,
    W: WriteNumber,
{
    let mut io = ComposedCpuIo {
        reader: input,
        writer: output,
    };
    execute_with_io(mem, &mut io)
}

pub fn execute(
    mem: &mut [i64],
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> Result<(), IntcodeError> {
    let mut input_trait_object = BufReadNumber { buf_read: input };
    let mut output_trait_object = WriteWriteNumber {
        output,
        prompt: true,
    };
    execute_composed(mem, &mut input_trait_object, &mut output_trait_object)
}

pub fn execute_with_std_io(mem: &mut [i64]) -> Result<(), IntcodeError> {
    execute(
        mem,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
    )
}

pub fn execute_no_prompt(
    mem: &mut [i64],
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> Result<(), IntcodeError> {
    let mut input_trait_object = BufReadNumber { buf_read: input };
    let mut output_trait_object = WriteWriteNumber {
        output,
        prompt: false,
    };
    execute_composed(mem, &mut input_trait_object, &mut output_trait_object)
}

pub fn execute_no_io(mem: &mut [i64]) -> Result<(), IntcodeError> {
    execute(mem, &mut std::io::empty(), &mut std::io::sink())
}

pub fn execute_with_channel(
    mem: &mut [i64],
    input: &Receiver<i64>,
    output: SyncSender<i64>,
) -> Result<(), IntcodeError> {
    let mut input_trait_object = ChannelReadNumber { input };
    let mut output_trait_object = ChannelWriteNumber { output };
    execute_composed(mem, &mut input_trait_object, &mut output_trait_object)
}

struct AsciiCpuState<'a, IO>
where
    IO: AsciiCpuIo,
{
    io: &'a mut IO,
    program_input_bytes: Vec<u16>,
    program_output_bytes: Vec<u8>,
}

impl<IO> AsciiCpuState<'_, IO>
where
    IO: AsciiCpuIo,
{
    fn create(io: &mut IO) -> AsciiCpuState<'_, IO> {
        AsciiCpuState {
            io,
            program_input_bytes: Vec::new(),
            program_output_bytes: Vec::new(),
        }
    }
}

impl<IO> CpuIo for AsciiCpuState<'_, IO>
where
    IO: AsciiCpuIo,
{
    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        Ok(())
    }

    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        if self.program_input_bytes.is_empty() {
            // We put the bytes in reverse order so we can pop them out.
            self.program_input_bytes.push('\n' as u16);
            let output_string = self.io.get_input_line_for_program()?;
            let output_string: Vec<_> = output_string.encode_utf16().collect();
            for ch in output_string.into_iter().rev() {
                self.program_input_bytes.push(ch);
            }
        }
        Ok(self.program_input_bytes.pop().unwrap() as i64)
    }

    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
        if num == '\n' as i64 {
            let s: String = String::from_utf8(self.program_output_bytes.clone()).unwrap();
            self.program_output_bytes.clear();
            self.io.accept_output_line_from_program(&s)?;
        } else {
            self.program_output_bytes.push(num as u8);
        }
        Ok(())
    }
}

pub fn execute_with_ascii_io<IO>(mem: &mut [i64], io: &mut IO) -> Result<(), IntcodeError>
where
    IO: AsciiCpuIo,
{
    let mut ascii_state = AsciiCpuState::create(io);
    let mut cpu = CpuState::create(mem, &mut ascii_state);
    if let Err(err) = cpu.execute() {
        Err(err)
    } else {
        //Copy the changed memory back into the input array
        for (i, value) in mem.iter_mut().enumerate() {
            *value = cpu.load_raw(i.try_into().unwrap())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::sync_channel;

    //Takes a single number, x, as input. Outputs one number under depending on x:
    //  x <  8 => 999
    //  x == 8 => 1000
    //  x >  8 => 1001
    const AROUND_EIGHT : &str = "3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99";

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

        test_io_program(AROUND_EIGHT, "0\n", "Please enter a number: 999\n");
        test_io_program(AROUND_EIGHT, "1\n", "Please enter a number: 999\n");
        test_io_program(AROUND_EIGHT, "3\n", "Please enter a number: 999\n");
        test_io_program(AROUND_EIGHT, "7\n", "Please enter a number: 999\n");
        test_io_program(AROUND_EIGHT, "8\n", "Please enter a number: 1000\n");
        test_io_program(AROUND_EIGHT, "9\n", "Please enter a number: 1001\n");
        test_io_program(AROUND_EIGHT, "10\n", "Please enter a number: 1001\n");
        test_io_program(AROUND_EIGHT, "42\n", "Please enter a number: 1001\n");
    }

    fn test_channel_io_helper(input: i64, expected_output: i64) {
        let mut mem = parse_program(AROUND_EIGHT).expect("failed to parse input");
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

    #[test]
    fn test_channel_io_closed_input() {
        let mut mem = parse_program(AROUND_EIGHT).expect("failed to parse input");
        let (input_send, input_recv) = sync_channel(1);
        let (output_send, output_recv) = sync_channel(1);
        drop(input_send);
        match execute_with_channel(&mut mem, &input_recv, output_send) {
            Err(IntcodeError::RecvError(_)) => {}
            other => panic!("unexpected result: {:?}", other),
        }
        output_recv.recv().expect_err("expected error on recv");
    }

    #[test]
    fn test_channel_io_closed_output() {
        let mut mem = parse_program(AROUND_EIGHT).expect("failed to parse input");
        let (input_send, input_recv) = sync_channel(1);
        let (output_send, output_recv) = sync_channel(1);
        drop(output_recv);
        input_send.send(0).expect("failed to send input");
        match execute_with_channel(&mut mem, &input_recv, output_send) {
            Err(IntcodeError::SendError(_)) => {}
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_large_number() {
        //Math
        test_io_program(
            "1102,34915192,34915192,7,4,7,99,0",
            "",
            "1219070632396864\n",
        );
        //IO
        test_io_program("104,1125899906842624,99", "", "1125899906842624\n");
    }

    #[test]
    fn test_relative_base() {
        //Quine
        test_io_program(
            "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99",
            "",
            "109\n1\n204\n-1\n1001\n100\n1\n100\n1008\n100\n16\n101\n1006\n101\n0\n99\n",
        );
    }
}
