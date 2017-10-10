use std::env;
use std::io;
use std::io::Write;
use std::io::Read;
use std::io::BufRead;
use std::process;
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
        process::exit(1);
    }
    let port: u32 = match args[2].parse() {
        Ok(p) => p,
        Err(_) => { println!("invalid port number"); process::exit(1); }
    };
    let host = &args[1];
    let ending = "\x0A";
    ///////////////////////////////////////////////////////////////

    // Pick a token that will not be used by any other socket and
    // use that one for the listener.
    const SOCKREADER: Token = Token(0);

    // The `Poll` instance
    let poll = Poll::new().unwrap();

    let addr = match format!("{}:{}", host, port).parse() {
        Ok(a) => { a },
        //FIXME allow actual host names, not just IP addresses
        Err(e) => { println!("Couldn't parse address {}:{} ({}).", host, port, e); process::exit(1); }
    };

    let mut stream = TcpStream::connect(&addr).unwrap();
    let mut write_stream = stream.try_clone().expect("cloning failed, yikes");

    // Register the listener
    poll.register(&stream,
                  SOCKREADER,
                  Ready::readable(), // andere events wären toll, z.b. disconnect
                  PollOpt::edge()).unwrap();

    // Event storage
    let mut events = Events::with_capacity(1024);

    // Read buffer, this will never actually get filled
    let mut buf = [0; 256];

    thread::spawn(move || {
        loop {
            let input = std::io::stdin();
            for line in input.lock().lines() {
                let l = line.unwrap() + ending; // this could be an invalid string.
                let bytes = l.as_bytes();
                let byte_length = bytes.len();
                let written = write_stream.write(bytes).unwrap(); // dont unwrap, could be broken pipe
                assert_eq!(byte_length, written);
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
