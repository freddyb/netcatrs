use std::env;
use std::io;
use std::io::Write;
use std::io::Read;
use std::io::BufRead;
use std::process;
use std::str;
//use std::sync::mpsc::{Sender, Receiver};
//use std::sync::mpsc;
use std::thread;

extern crate mio;
use mio::{Events, Ready, Poll, PollOpt, Token};
use mio::net::TcpStream;

//use std::collections::HashMap;


fn main() {
    // arg parsing
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <ip> <port>", args[0]);
    }
    let port: u32 = args[2].parse().expect("Invalid Port");
    let ref host = args[1];

    ///////////////////////////////////////////////////////////////

    // Pick a token that will not be used by any other socket and
    // use that one for the listener.
    const SOCKREADER: Token = Token(0);
    const SOCKWRITER: Token = Token(1);
    const STDINREADER: Token = Token(2);

    // The `Poll` instance
    let poll = Poll::new().unwrap();

    let addr = format!("{}:{}", host, port).parse().unwrap(); //TODO catch parse failure

    let mut stream = TcpStream::connect(&addr).unwrap();
    let mut write_stream = stream.try_clone().expect("cloning failed, yikes");

    // Register the listener
    poll.register(&stream,
                  SOCKREADER,
                  Ready::readable(),
                  PollOpt::edge()).unwrap();
    //poll.register(&stream, SOCKWRITER, Writer::writable(), PollOpt::edge())?;


    // Event storage
    let mut events = Events::with_capacity(1024);

    // Read buffer, this will never actually get filled
    let mut buf = [0; 256];

    let stdin_thread = thread::spawn(move || {
        loop {
            let input = std::io::stdin();
            for line in input.lock().lines() {
                let l = line.unwrap(); // this could be an invalid string.
                let bytes = l.as_bytes();
                let byte_length = bytes.len();
                let written = write_stream.write(bytes).unwrap();
                assert_eq!(byte_length, written);
                write_stream.write("\x0A".as_bytes()); // TODO allow -C switch to write 0A0D
            }
        }
    });

    // The main event loop
    loop {
        // Wait for events
        poll.poll(&mut events, None).expect("Can't poll. Aborting.");

        for event in &events {
            match event.token() {
                SOCKREADER => {
                    // Continue reading in a loop until `WouldBlock` is
                    // encountered.
                    loop {
                        match stream.read(&mut buf) {
                            Ok(read_len) => {
                                println!("{:?}", String::from_utf8_lossy(&buf[0..read_len]));
                                break;
                            }
                            // Data is not actually sent in this example
                            //Ok(_) => unreachable!(),
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                // Socket is not ready anymore, stop reading
                                break;
                            },
                            Err(ref e) if e.kind() == io::ErrorKind::ConnectionRefused => {
                                println!("connect to {} port {} (tcp) failed: Connection refused", host, port);
                                process::exit(1);
                            }
                            e => panic!("err={:?}", e), // Unexpected error, e.g., connection refused
                        }
                    }
                },
                x => panic!("unexpected token {:?}", x),
            }
        }
    }
}
