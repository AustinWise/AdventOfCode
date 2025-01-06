use std::error::Error;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::sync::LazyLock;
use std::thread;

use intcode::CpuIo;
use intcode::IntcodeError;

static PROGRAM: LazyLock<Vec<i64>> =
    LazyLock::new(|| intcode::parse_program(include_str!("input.txt")).unwrap());

#[derive(Debug)]
struct Message {
    address: i64,
    x: i64,
    y: i64,
}

enum BuffingMessageToSend {
    None,
    HaveOne { address: i64 },
    HaveTwo { address: i64, x: i64 },
}

struct ComputerIo {
    address: i64,
    inbox: Receiver<Message>,
    outbox: Sender<Message>,
    incoming_buffer: Option<i64>,
    partial_outgoing_message: BuffingMessageToSend,
}

impl CpuIo for ComputerIo {
    fn prompt_for_number(&mut self) -> Result<(), IntcodeError> {
        Ok(())
    }

    fn read_number(&mut self) -> Result<i64, IntcodeError> {
        if let Some(num) = self.incoming_buffer {
            self.incoming_buffer = None;
            Ok(num)
        } else {
            match self.inbox.try_recv() {
                Ok(message) => {
                    assert_eq!(self.address, message.address);
                    self.incoming_buffer = Some(message.y);
                    Ok(message.x)
                }
                Err(TryRecvError::Empty) => Ok(-1),
                Err(TryRecvError::Disconnected) => Err(IntcodeError::UserInitiatedExit),
            }
        }
    }

    fn write_number(&mut self, num: i64) -> Result<(), IntcodeError> {
        self.partial_outgoing_message = match self.partial_outgoing_message {
            BuffingMessageToSend::None => BuffingMessageToSend::HaveOne { address: num },
            BuffingMessageToSend::HaveOne { address } => {
                BuffingMessageToSend::HaveTwo { address, x: num }
            }
            BuffingMessageToSend::HaveTwo { address, x } => {
                let message = Message { address, x, y: num };
                if self.outbox.send(message).is_err() {
                    return Err(IntcodeError::UserInitiatedExit);
                }
                BuffingMessageToSend::None
            }
        };
        Ok(())
    }
}

fn one_computer(address: i64, inbox: Receiver<Message>, outbox: Sender<Message>) {
    let mut mem = PROGRAM.clone();
    let mut io = ComputerIo {
        address,
        inbox,
        outbox,
        // the first input command gives the computer its address,
        incoming_buffer: Some(address),
        partial_outgoing_message: BuffingMessageToSend::None,
    };
    intcode::execute_with_io(&mut mem, &mut io).unwrap();
}

fn part_1() {
    let mut computer_inboxes = Vec::new();
    let mut thread_handles = Vec::new();
    let (switch_sender, switch_receiver) = channel::<Message>();
    for address in 0..50 {
        let (tx, rx) = channel::<Message>();
        computer_inboxes.push(tx);

        let switch_sender = switch_sender.clone();
        thread_handles.push(thread::spawn(move || {
            one_computer(address, rx, switch_sender)
        }));
    }

    drop(switch_sender);

    loop {
        let message = switch_receiver.recv().unwrap();
        match message.address {
            0..50 => {
                computer_inboxes[message.address as usize]
                    .send(message)
                    .unwrap();
            }
            255 => {
                println!("got part 1 message: {:?}", message);
                println!("part 1: {}", message.y);
                break;
            }
            _ => panic!("unexpected address: {}", message.address),
        }
    }

    drop(computer_inboxes);
    drop(switch_receiver);

    for thread in thread_handles {
        thread.join().unwrap();
    }

    println!("all computer threads exited");
}

fn main() -> Result<(), Box<dyn Error>> {
    part_1();

    println!("{}", PROGRAM.len());
    Ok(())
}
