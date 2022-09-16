use std::io::{BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str;

use log::debug;
use threadpool::ThreadPool;

const NUM_WORKERS: usize = 5;

pub fn run_server() {
    let listener = TcpListener::bind("0.0.0.0:5001").unwrap();

    let pool = ThreadPool::new(NUM_WORKERS);
    for stream in listener.incoming() {
        pool.execute(move || handle_connection(stream.unwrap()));
    }
}

fn handle_connection(mut stream: TcpStream) {
    debug!("Handling a connection.");

    debug!("Reading bytes sent by client.");
    // Read bytes from the stream.
    let mut reader = BufReader::new(&stream);
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();

    debug!("Writing bytes back.");
    // Write them back, verbatim.
    stream.write_all(&buf).unwrap();

    debug!("Connection handled.");
}

pub fn run_client(url: Option<String>, bytes: &[u8]) {
    debug!(
        "Writing {} to server.",
        String::from_utf8(bytes.to_vec()).unwrap()
    );
    let url = match url {
        Some(url_string) => url_string,
        None => "127.0.0.1:5001".to_string(),
    };

    let mut stream = TcpStream::connect(url).unwrap();
    stream.write_all(bytes).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    debug!("Bytes written.");

    debug!("Reading bytes from server.");
    let mut reader = BufReader::new(&stream);
    let mut buf = vec![];
    let _bytes_read = reader.read_to_end(&mut buf).unwrap();
    debug!("Bytes read: {}", str::from_utf8(&buf).unwrap());
}
