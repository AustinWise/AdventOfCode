use std::error::Error;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

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
            // Block a tiny bit before returning -1. Without this each "computer" consumes a lot of
            // real CPU spinning.
            match self.inbox.recv_timeout(Duration::from_millis(1)) {
                Ok(message) => {
                    assert_eq!(self.address, message.address);
                    self.incoming_buffer = Some(message.y);
                    Ok(message.x)
                }
                Err(RecvTimeoutError::Timeout) => Ok(-1),
                Err(RecvTimeoutError::Disconnected) => Err(IntcodeError::UserInitiatedExit),
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

fn run_computer_network(enable_nat: bool) {
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

    let mut nat_value: Option<Message> = None;
    let mut previously_sent_nat_y_value: Option<i64> = None;

    loop {
        // Ideally this would more preciously measure "idle": all the `ComputerIo` structs have
        // `incoming_buffer == None` and there are no messages in our switch channel.
        // Instead we assume that if we have not recieved any messages at the switch for some period
        // of time, the network is "idle". The larger the receive timeout, the more likly this is true.
        // It seems to work well enough 10 ms here and having each computer sleep 1 ms before considering
        // its incoming queue empty.
        let message = match switch_receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(message) => message,
            Err(RecvTimeoutError::Timeout) => {
                if enable_nat {
                    if let Some(nat_value) = &nat_value {
                        println!("sending nat value to address 0: {:?}", nat_value);
                        if let Some(prev) = previously_sent_nat_y_value {
                            if prev == nat_value.y {
                                println!("part 2 answer: {}", prev);
                                break;
                            }
                        }
                        previously_sent_nat_y_value = Some(nat_value.y);
                        computer_inboxes[0]
                            .send(Message {
                                address: 0,
                                x: nat_value.x,
                                y: nat_value.y,
                            })
                            .unwrap();
                    }
                }
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => panic!("switch disconnected?"),
        };
        match message.address {
            0..50 => {
                computer_inboxes[message.address as usize]
                    .send(message)
                    .unwrap();
            }
            255 => {
                if enable_nat {
                    nat_value = Some(message);
                } else {
                    println!("got part 1 message: {:?}", message);
                    println!("part 1: {}", message.y);
                    println!();
                    println!();
                    break;
                }
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
    run_computer_network(false);
    run_computer_network(true);
    Ok(())
}
