use clap::Parser;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread::{self, JoinHandle};
use std::time::Instant;

#[derive(Parser, Debug)]
pub struct Args {
    /// Number of simultaneous connections
    #[arg(short, long, default_value_t = 1)]
    pub connections: usize,
    /// Number of requests in every connection
    #[arg(short, long, default_value_t = 1)]
    pub requests: usize,
    /// If to check writing to connection only
    #[arg(short, long, default_value_t = false)]
    pub write_only: bool,
    /// Endpoint to connect
    pub endpoint: String,
}

impl Args {
    pub fn parse_args() -> Args {
        Args::parse()
    }
}

pub fn run(args: Args) {
    println!("{:?}", args);

    let start = Instant::now();
    let all_requests = args.connections * args.requests;
    let mut successful_requests: usize = 0;

    let mut threads = Vec::with_capacity(args.connections);
    for i in 0..args.connections {
        let thread = new_thread(i, &args.endpoint, args.requests, args.write_only);
        threads.push(thread);
    }
    for thread in threads {
        if let Ok(Ok(n)) = thread.join() {
            successful_requests = successful_requests + n;
        }
    }

    let duration = start.elapsed();

    println!("============================");
    println!("RESULTS:");
    println!("execution duration: {:?}", duration);
    println!("planned requests: {all_requests}");
    println!("successful requests: {successful_requests}");
}

fn new_thread(
    id: usize,
    addr: &str,
    req_num: usize,
    write_only: bool,
) -> JoinHandle<Result<usize, std::io::Error>> {
    let addr = addr.to_string();
    thread::spawn(move || create_connection(id, &addr, req_num, write_only))
}

fn create_connection(
    id: usize,
    addr: &str,
    req_num: usize,
    write_only: bool,
) -> Result<usize, std::io::Error> {
    println!("creating connection #{id}");

    let mut stream = match TcpStream::connect(addr) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("error: cant't create connection {id}: {e}");
            return Err(e);
        }
    };

    let mut successful_requests: usize = 0;
    let write_message = b"ping\n";
    let mut read_buf = [0 as u8; 1024];

    for i in 0..req_num {
        println!("new request #{i} for connection #{id}");

        if let Err(e) = stream.write(write_message) {
            eprintln!("error: in conn #{id}: {e}");
            break;
        }

        // try to read from the stream only if write_only=false
        if !write_only {
            match stream.read(&mut read_buf) {
                Err(e) => {
                    eprintln!("error: in conn #{id}: {e}");
                    break;
                }
                Ok(0) => {
                    eprintln!("error: conn #{id}: read 0 bytes");
                    break;
                }
                Ok(_) => {}
            }
        }

        successful_requests += 1;
    }
    Ok(successful_requests)
}
