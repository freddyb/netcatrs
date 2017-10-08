use std::env;
use std::io;
use std::io::Write;
use std::io::Read;
use std::io::BufRead;
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

    let addr  = format!("{}:{}", host, port).parse().unwrap();

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

        // The main event loop
        loop {
            // Wait for events
            poll.poll(&mut events, None).expect("Can't poll. Aborting.");

            let input = std::io::stdin();
            for line in input.lock().lines() {
                let l = line.unwrap();
                let bytes = l.as_bytes();
                let byte_length= bytes.len();
                let written = write_stream.write(bytes).unwrap();
                assert_eq!(byte_length, written);
            }


            for event in &events {
                match event.token() {
                    SOCKREADER => {
                        // Continue reading in a loop until `WouldBlock` is
                        // encountered.
                        loop {
                            match stream.read(&mut buf) {
                                Ok(0) => {
                                    println!("{:?}", String::from_utf8_lossy(&buf));
                                    break;
                                }
                                // Data is not actually sent in this example
                                Ok(_) => unreachable!(),
                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    // Socket is not ready anymore, stop reading
                                    break;
                                }
                                e => panic!("err={:?}", e), // Unexpected error
                            }
                        }
                    },
                    x => panic!("unexpected token {:?}", x),

                }
            }
        }
    }
///////////////////////////////////////////////////////////////////////////////////////////////////
// channels for net thread
//    let (clitx, clirx) : (Sender<String>, Receiver<String>) = mpsc::channel();
//
//    let clithread = thread::spawn(move || {
//        loop {
//            // read from stdin and send to net-thread
//            let input = std::io::stdin();
//               for line in input.lock().lines() {
//                    clitx.send(line.unwrap());
//                }
//            // read from net-thread and print not needed, net-thread can print itself
//        }
//    });
//
//    // net thread
//    let netthread = thread::spawn(move || {
//        let ref host = args[1];
//        if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", host, port)) {
//            // read won't block indefinitely
//            println!("connected.");
//            loop {
//                // read from socket and print
//
//                /*let mut lineinprogress: Vec<u8> = Vec::new();
//
//                let mut buffer = [0; 10];
//                let mut i = 0;
//                stream.read_exact(&mut buffer);
//                for byte in buffer.iter() {
//                    i += 1;
//                    if byte == &0x0a {
//                        lineinprogress.extend(&buffer[0 .. i]);
//                        print!("{:?}", lineinprogress);
//                    } else {
//                        lineinprogress.extend(buffer.iter());
//                    }
//                }*/
//                //stream.read_to_string(&mut buf);
//
//                // read from cli-thread and send to socket
//
//                match clirx.try_recv() {
//                    Ok(line) => {
//                        // need to find a way to check if socket still alive
//                        stream.write(line.as_bytes());
//                        stream.write(&[0x0A]);
//                    }
//                    Err(error) => {
//                        // pass
//                    }
//                }
//            }
//        } else {
//            println!("couldnt connect to stream");
//            // TODO figure out how to end here
//        }
//    });
//
//    netthread.join();
//    clithread.join();
//}

