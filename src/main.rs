use std::io::Write;
use std::net::{IpAddr, TcpStream};
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::{env, io, process, thread};

const MAX: u16 = 65535;

struct Arguments {
    flag: String,
    ip_address: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        } else if args.len() > 4 {
            return Err("too many arguments");
        }

        let f = args[1].clone();
        return if let Ok(ip_address) = IpAddr::from_str(&f) {
            Ok(Arguments {
                flag: String::from(""),
                ip_address,
                threads: 4,
            })
        } else {
            let flag = args[1].clone();

            if flag.contains("-h") || flag.contains("-help") && args.len() == 2 {
                println!(
                    "usage: -j to select how many threads you want\n\
                -h or -help to show this help message"
                );

                Err("help")
            } else if flag.contains("-h") || flag.contains("-help") {
                Err("too many arguments")
            } else if flag.contains("-j") {
                let ip_address = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("not a valid IP address. Must be Ipv4 or IPv6"),
                };

                let threads = match args[2].parse::<u16>() {
                    Ok(s) => s,
                    Err(_) => return Err("failed to parse thread number"),
                };

                Ok(Arguments {
                    threads,
                    flag,
                    ip_address,
                })
            } else {
                Err("invalid syntax")
            }
        };
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;

    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        }

        if (MAX - port) <= num_threads {
            break;
        }

        port += num_threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(1);
        } else {
            eprintln!("{program} problem parsing arguments: {err}");
            process::exit(2);
        }
    });

    let num_threads = arguments.threads;
    let address = arguments.ip_address;
    let (tx, rx) = channel();
    for i in 0..num_threads {
        let tx = tx.clone();

        thread::spawn(move || {
            scan(tx, i, address, num_threads);
        });
    }

    let mut out = vec![];
    drop(tx);

    for p in rx {
        out.push(p);
    }

    println!();
    out.sort();
    for v in out {
        println!("{v} is open");
    }
}
