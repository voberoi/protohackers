use std::io::{BufReader, Read, Write};
use std::net::TcpStream;

pub fn handle_connection(mut stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut buf = vec![];
    reader.read_to_end(&mut buf).unwrap();
    stream.write_all(&buf).unwrap();
}

#[cfg(test)]
mod test {
    #[test]
    fn test_happy_path() {}
}
