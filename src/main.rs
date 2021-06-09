use std::process;
use std::error::Error;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, SystemTime};

use clap::Clap;
use std::convert::TryFrom;

#[derive(Clap)]
#[clap(name = "tcping")]
struct Args {
    #[clap(short = 'n', long, default_value = "4")]
    count: u8,

    #[clap(short, long, default_value = "1")]
    interval: f32,

    #[clap(short = 'w', long, default_value = "2")]
    timeout: f32,

    #[clap(short = '4')]
    ipv4: bool,
    #[clap(short = '6')]
    ipv6: bool,

    hostname: String,

    #[clap(default_value = "80")]
    port: String,
}

#[derive(PartialEq)]
enum IpAddrKind {
    IPv4,
    IPv6,
    Any,
}

fn main() {
    let args = Args::parse();
    if args.count < 1 {
        eprintln!("count must be an positive integer");
        process::exit(1);
    }
    let ip_version = match (args.ipv4, args.ipv6) {
        (true, true) => {
            eprintln!("ipv4 and ipv6 cannot be specified at same time");
            process::exit(1);
        }
        (true, false) => IpAddrKind::IPv4,
        (false, true) => IpAddrKind::IPv6,
        (false, false) => IpAddrKind::Any,
    };

    let socket = match solve_address(&args.hostname, &args.port, ip_version) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    let timeout_duration = Duration::from_secs_f32(args.timeout);
    for i in 0..args.count {
        handle_tcping(&socket, timeout_duration);
        if i != (args.count - 1) {
            thread::sleep(Duration::from_secs_f32(args.interval));
        }
    }
}

fn solve_address(host: &str, port: &str, ip_version: IpAddrKind) -> Result<SocketAddr, Box<dyn Error>> {
    let mut address = host.to_owned();
    if !address.contains(':') {
        address.push(':');
        address.push_str(port);
    };
    // Ok(address.to_socket_addrs()?.next().ok_or_else(Err(""))?)
    for address in address.to_socket_addrs()? {
        if (ip_version == IpAddrKind::IPv4) && address.is_ipv6() { continue; }
        if (ip_version == IpAddrKind::IPv6) && address.is_ipv4() { continue; }
        return Ok(address);
    }
    Err(Box::try_from("cannot resolve hostname").unwrap())
}

fn handle_tcping(sockaddr: &SocketAddr, timeout: Duration) {
    let sys_time = SystemTime::now();

    let result = TcpStream::connect_timeout(sockaddr, timeout);
    let duration =
        SystemTime::now()
            .duration_since(sys_time)
            .unwrap()
            .as_millis();
    match result {
        Ok(_) => {
            println!("connected to {} {}ms", sockaddr, duration);

            // Ok(stream)
        }
        Err(err) => match err.kind() {
            ErrorKind::TimedOut => {
                println!("connected to {} timeout {}ms", sockaddr, duration);
                // Err(err)
            }
            _ => {
                println!("connected to {} failed {}ms", sockaddr, duration);
                // Err(err)
            }
        },
    };
}
