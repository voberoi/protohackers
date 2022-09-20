use global_counter::global_counter;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

global_counter!(PORT_COUNTER, u32, 5001);

pub struct ServerProcess {
    port: String,
    process: Child,
}

enum ServerType {
    PrimeTime,
}

impl ServerProcess {
    pub fn run_prime_time() -> Self {
        ServerProcess::run(ServerType::PrimeTime)
    }

    // Runs a server process of the given type and waits to be able to connect
    // to it before returning.
    fn run(server_type: ServerType) -> Self {
        let port = &(PORT_COUNTER.inc_cloning().to_string());

        let mut cargo_args = vec!["run", "--", "-p", port];
        let mut args = match server_type {
            ServerType::PrimeTime => vec!["prime-time"],
        };
        cargo_args.append(&mut args);

        let child = Command::new("cargo").args(cargo_args).spawn().unwrap();

        println!("({}) Waiting for server to start.", port);
        let server = ServerProcess {
            port: port.to_string(),
            process: child,
        };
        server.wait_until_started();
        println!("({}) Server started.", port);

        server
    }

    pub fn get_stream(&self) -> TcpStream {
        let conn = TcpStream::connect(self.url()).unwrap();
        conn.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        conn.set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        conn
    }

    // Sends the given bytes to the server and returns the bytes received from
    // the server in response.
    pub fn send_request(&self, bytes: &[u8]) -> Vec<u8> {
        let mut stream = TcpStream::connect(self.url()).unwrap();

        stream.write_all(bytes).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();

        let mut reader = BufReader::new(&stream);
        let mut buf = vec![];
        let _bytes_reader = reader.read_to_end(&mut buf).unwrap();
        buf
    }

    fn wait_until_started(&self) {
        while let Err(_e) = TcpStream::connect(self.url()) {
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn url(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        println!("({}) Killing child process.", self.port);
        self.process.kill().unwrap();
        println!(
            "({}) Sent kill signal successfully. Waiting until killed...",
            self.port
        );
        self.process.wait().unwrap(); // Wait until killed.
        println!("({}) Child process killed.", self.port);
    }
}

pub fn connection_is_open(conn: &TcpStream) -> bool {
    let mut buf = [0; 10];
    conn.set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();

    match conn.peek(&mut buf) {
        Ok(0) => {
            // EOF -- closed.
            false
        }
        Ok(_) => {
            // Can read bytes. Connection still open.
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            // No bytes available. Read would block but connection is open.
            true
        }
        Err(_) => {
            // Some other error, connection is closed.
            false
        }
    }
}

pub fn write_line(stream: &mut TcpStream, content: String) {
    let mut body = content.clone();
    body.push('\n');
    stream.write_all(body.as_bytes()).unwrap();
}

pub fn write_json_line(stream: &mut TcpStream, obj: &json::JsonValue) {
    let mut jsonl = obj.dump();
    jsonl.push('\n');
    stream.write_all(jsonl.as_bytes()).unwrap();
}

pub fn read_line(stream: &mut TcpStream) -> String {
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    reader.read_line(&mut buf).unwrap();
    buf
}
