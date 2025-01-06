use std::convert::TryInto;
use std::error::Error;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;

use intcode::Memory;

//TODO: make this not allocate so much?
//TODO: make this a generator?
fn permutation(begin: i64, number_of_perms: u64) -> Vec<Vec<i64>> {
    fn inner(result: &mut Vec<Vec<i64>>, prefix: &Vec<i64>, to_perm: &Vec<i64>) {
        if to_perm.is_empty() {
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

    let mut to_perm: Vec<i64> = Vec::new();
    for i in 0..number_of_perms {
        let signed: i64 = i.try_into().unwrap();
        to_perm.push(begin + signed);
    }

    let mut ret = Vec::new();
    if number_of_perms != 0 {
        inner(&mut ret, &Vec::new(), &to_perm);
    }
    ret
}

fn run_amplifier_controller_program(
    program: &Memory,
    phase_setting: &Vec<i64>,
) -> Result<i64, Box<dyn Error>> {
    let mut input: i64 = 0;
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

fn find_max_thruster(program: &Memory) -> Result<i64, Box<dyn Error>> {
    let mut max_thrust = i64::MIN;

    for phase_setting in permutation(0, 5) {
        max_thrust = max_thrust.max(run_amplifier_controller_program(program, &phase_setting)?);
    }

    Ok(max_thrust)
}

fn run_amplifier_controller_program_part(
    program: Memory,
    input: Receiver<i64>,
    output: SyncSender<i64>,
) -> Result<(), intcode::IntcodeError> {
    let mut mem = program;
    intcode::execute_with_channel(&mut mem, &input, output)
}

fn pump_feedback(
    input: Receiver<i64>,
    output: SyncSender<i64>,
) -> Result<i64, intcode::IntcodeError> {
    let mut res = None;
    loop {
        let num = match input.recv() {
            Ok(num) => num,
            Err(_) => return Ok(res.unwrap()),
        };
        res = Some(num);
        //We want the last value sent from the Receiver. So ignore sending errors.
        //It should not be possible for the first amplifier thread to shut down
        //before the last value is produced on the amplifier. But we ignore errors
        //just in case that assumption is not true.
        let _ = output.send(num);
    }
}

fn run_amplifier_controller_program_feedback(
    program: &Memory,
    phase_setting: &[i64],
) -> Result<i64, intcode::IntcodeError> {
    let (feedback_front_send, feedback_front_recv) = sync_channel::<i64>(10);
    let (send1, recv1) = sync_channel::<i64>(10);
    let (send2, recv2) = sync_channel::<i64>(10);
    let (send3, recv3) = sync_channel::<i64>(10);
    let (send4, recv4) = sync_channel::<i64>(10);
    let (feedback_back_send, feedback_back_recv) = sync_channel::<i64>(10);
    feedback_front_send.send(phase_setting[0])?;
    feedback_front_send.send(0)?;
    send1.send(phase_setting[1])?;
    send2.send(phase_setting[2])?;
    send3.send(phase_setting[3])?;
    send4.send(phase_setting[4])?;
    let mem1 = program.to_owned();
    let mem2 = program.to_owned();
    let mem3 = program.to_owned();
    let mem4 = program.to_owned();
    let mem5 = program.to_owned();

    let jh1 = thread::spawn(move || {
        run_amplifier_controller_program_part(mem1, feedback_front_recv, send1)
    });
    let jh2 = thread::spawn(move || run_amplifier_controller_program_part(mem2, recv1, send2));
    let jh3 = thread::spawn(move || run_amplifier_controller_program_part(mem3, recv2, send3));
    let jh4 = thread::spawn(move || run_amplifier_controller_program_part(mem4, recv3, send4));
    let jh5 = thread::spawn(move || {
        run_amplifier_controller_program_part(mem5, recv4, feedback_back_send)
    });
    let pumper = thread::spawn(move || pump_feedback(feedback_back_recv, feedback_front_send));

    jh1.join().unwrap()?;
    jh2.join().unwrap()?;
    jh3.join().unwrap()?;
    jh4.join().unwrap()?;
    jh5.join().unwrap()?;

    pumper.join().unwrap()
}

fn find_max_thruster_feedback(program: &Memory) -> Result<i64, Box<dyn Error>> {
    let mut max_thrust = i64::MIN;

    for phase_setting in permutation(5, 5) {
        let this_thrust = run_amplifier_controller_program_feedback(program, &phase_setting)?;
        max_thrust = max_thrust.max(this_thrust);
    }

    Ok(max_thrust)
}

fn main() -> Result<(), Box<dyn Error>> {
    let program = intcode::parse_program(&std::fs::read_to_string("input.txt")?)?;

    println!("max thrust - part1: {}", find_max_thruster(&program)?);
    println!(
        "max thrust - part2: {}",
        find_max_thruster_feedback(&program)?
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_perm() {
        let empty: Vec<Vec<i64>> = Vec::new();
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
        let empty: Vec<Vec<i64>> = Vec::new();
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

    #[test]
    fn test_find_max_thrust_feedback() {
        let program = intcode::parse_program(
            "3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5",
        )
        .expect("parse program");
        assert_eq!(
            139629729,
            find_max_thruster_feedback(&program).expect("find max thrust")
        );

        let program = intcode::parse_program(
            "3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10",
        )
        .expect("parse program");
        assert_eq!(
            18216,
            find_max_thruster_feedback(&program).expect("find max thrust")
        );
    }
}
