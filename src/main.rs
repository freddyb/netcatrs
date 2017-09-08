use std::env;
use std::io;
use std::io::Write;
use std::io::Read;
use std::io::BufRead;
use std::net::TcpStream;
use std::ops::Add;
use std::str;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;


fn main() {

    // arg parsing
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <ip> <port>", args[0]);
    }
    let port: u32 = args[2].parse().expect("Invalid Port");

    // channels for net thread
    let (clitx, clirx) : (Sender<String>, Receiver<String>) = mpsc::channel();

    let clithread = thread::spawn(move || {
        loop {
            // read from stdin and send to net-thread
            let input = std::io::stdin();
               for line in input.lock().lines() {
                    clitx.send(line.unwrap());
                }
            // read from net-thread and print not needed, net-thread can print itself
        }
    });

    // net thread
    let netthread = thread::spawn(move || {
        let ref host = args[1];
        if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", host, port)) {
            // read won't block indefinitely
            println!("connected.");
            loop {
                // read from socket and print

                /*let mut lineinprogress: Vec<u8> = Vec::new();

                let mut buffer = [0; 10];
                let mut i = 0;
                stream.read_exact(&mut buffer);
                for byte in buffer.iter() {
                    i += 1;
                    if byte == &0x0a {
                        lineinprogress.extend(&buffer[0 .. i]);
                        print!("{:?}", lineinprogress);
                    } else {
                        lineinprogress.extend(buffer.iter());
                    }
                }*/
                //stream.read_to_string(&mut buf);

                // read from cli-thread and send to socket
                match clirx.try_recv() {
                    Ok(line) => {
                        // need to find a way to check if socket still alive
                        stream.write(line.as_bytes());
                        stream.write(&[0x0A]);
                    }
                    Err(error) => {
                        // pass
                    }
                }
            }
        } else {
            println!("couldnt connect to stream");
            // TODO figure out how to end here
        }
    });

    netthread.join();
    clithread.join();
}

