use std::error::Error;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::net::{SocketAddr, ToSocketAddrs};
use std::process;
use std::thread;
use std::time::{Duration, SystemTime};

use clap::Parser;

#[derive(Parser)]
#[command()]
struct Args {
    /// Number of pings to send
    #[arg(short = 'c', long, default_value = "4")]
    count: u32,

    /// Ping forever
    #[arg(short = 't', long, default_value = "false", conflicts_with = "count")]
    forever: bool,

    /// Interval between pings
    #[arg(short, long, default_value = "1")]
    interval: f32,

    /// Timeout for each ping
    #[arg(short = 'w', long, default_value = "2")]
    timeout: f32,

    /// Use IPv4
    #[arg(short = '4', conflicts_with = "ipv6")]
    ipv4: bool,
    /// Use IPv6
    #[arg(short = '6')]
    ipv6: bool,

    /// Hostname or IP address
    hostname: String,

    /// Port number
    #[arg(default_value = "80")]
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
    if args.forever {
        loop {
            handle_tcping(&socket, timeout_duration);
            thread::sleep(Duration::from_secs_f32(args.interval));
        }
    }
    for i in 0..args.count {
        handle_tcping(&socket, timeout_duration);
        if i != (args.count - 1) {
            thread::sleep(Duration::from_secs_f32(args.interval));
        }
    }
}

fn solve_address(
    host: &str,
    port: &str,
    ip_version: IpAddrKind,
) -> Result<SocketAddr, Box<dyn Error>> {
    // check if host contains exactly one ":"
    let mut colon_count = 0;
    for c in host.chars() {
        if c == ':' {
            colon_count += 1;
            if colon_count > 1 {
                break;
            }
        }
    }

    let (host_local, port_local) = if colon_count == 1 {
        // split host and port
        let position = host.find(':').unwrap();
        let h = &host[0..position];
        let p = &host[position + 1..];
        // let mut split = host.split(":");
        // let h = split.next().unwrap();
        // let p = split.next().unwrap();
        (h, p)
    } else if host.contains("]:") {
        let position = host.rfind(':').unwrap();
        let h = &host[1..position - 1]; // remove '[' and ']'
        let p = &host[position + 1..];
        (h, p)
    } else if host.len() >= 2
        && host.as_bytes()[0] == b'['
        && host.as_bytes()[host.len() - 1] == b']'
    {
        let h = &host[1..host.len() - 1];
        (h, port)
    } else {
        (host, port)
    };

    let port_num: u16 = port_local.parse()?;

    for socket in (host_local, port_num).to_socket_addrs()? {
        if (ip_version == IpAddrKind::IPv4) && socket.is_ipv6() {
            continue;
        }
        if (ip_version == IpAddrKind::IPv6) && socket.is_ipv4() {
            continue;
        }
        return Ok(socket);
    }
    Err(Box::new(std::io::Error::new(
        ErrorKind::Other,
        "cannot resolve hostname",
    )))
}

fn handle_tcping(sockaddr: &SocketAddr, timeout: Duration) {
    let sys_time = SystemTime::now();

    let result = TcpStream::connect_timeout(sockaddr, timeout);
    let duration = SystemTime::now()
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
