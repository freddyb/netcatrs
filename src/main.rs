use std::env;
use std::io;
use std::io::Write;
use std::io::Read;
use std::io::BufRead;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process;
use std::thread;
extern crate getopts;
use getopts::Options;

extern crate mio;
use mio::{Events, Ready, Poll, PollOpt, Token};
use mio::net::TcpStream;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-46Chv] [destination] [port]", program);
    print!("{}", opts.usage(&brief));
}
fn main() {
    // arg parsing
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("v", "verbose", "give more verbose output");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("C", "crlf", "Send CRLF as line ending");
    opts.optflag("4", "four", "use IPv64 (default)");
    opts.optflag("6", "six", "use IPv6");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(e) => { panic!(e.to_string()) }
    };
    // parse flags
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if matches.opt_present("v") {
        let verbose = true;
    } else {
        let verbose = false;
    }
    let ending : &str;
    if matches.opt_present("C") {
        ending = "\x0D\x0A";
    } else {
        ending = "\x0A";
    }
    let mut v6 = false;
    if matches.opt_present("6") {
        v6 = true;
    }

    // parse hots/port
    if !matches.free.len() == 2 {
        print_usage(&program, opts);
        return;
    };
    let host = matches.free[0].clone();
    let port: u32 = match matches.free[1].parse() {
        Ok(p) => p,
        Err(_) => { println!("invalid port number"); process::exit(1); }
    };



    ///////////////////////////////////////////////////////////////

    // Pick a token that will not be used by any other socket and
    // use that one for the listener.
    const SOCKREADER: Token = Token(0);
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
                                match io::stdout().write(&buf[0..read_len]) {
                                    Ok(len) => { assert_eq!(len, read_len) },
                                    Err(e) => { panic!("{:?}", e); }
                                 }
                                break;
                            }
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                // Socket is not ready anymore, stop reading
                                break;
                            },
                            Err(ref e) if e.kind() == io::ErrorKind::ConnectionRefused => {
                                println!("connect to {} port {} (tcp) failed: Connection refused", host, port);
                                process::exit(1);
                            },
                            Err(ref e)if e.raw_os_error().unwrap() == 113 => {
                                println!("connect to {} port {} (tcp) failed: No route to host", host, port);
                                process::exit(1);
                            },
                            e => panic!("err={:?}, ", e), // Other, unexpected error
                        }
                    }
                },
                x => panic!("unexpected token {:?}", x),
            }
        }
    }
}
