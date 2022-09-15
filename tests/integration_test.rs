// use assert2::*;
use granulator::*;

use std::sync::mpsc::channel;
use std::thread;

#[test]
fn multi_threading() {
    let (tx, rx) = channel();

    let sender = thread::spawn(move || {
        let g = manager::Granulator::new();
        tx.send(g).expect("Unable to send on channel");
    });

    let receiver = thread::spawn(move || rx.recv().expect("Unable to receive from channel"));

    sender.join().expect("The sender thread has panicked");
    let _sended = receiver.join().expect("The receiver thread has panicked");
}
