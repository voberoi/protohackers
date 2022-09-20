use json::object;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use thiserror::Error;

#[derive(Debug, Error)]
enum PrimeTimeError {
    #[error("Invalid JSON request.")]
    InvalidRequest,
}

pub fn handle_connection(mut stream: TcpStream) {
    let mut read_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&mut read_stream);

    loop {
        let mut buf = String::new();

        match reader.read_line(&mut buf) {
            Ok(0) => {
                // EOF -- connection closed. No-op.
                break;
            }
            Ok(_) => match json::parse(&buf.clone()) {
                // We read a line.
                Ok(p) => match validate_request(p) {
                    Ok(r) => {
                        let mut response_string = r.dump();
                        response_string.push('\n');
                        stream.write_all(response_string.as_bytes()).unwrap();
                    }
                    Err(_e) => {
                        write_malformed_response(&mut stream);
                        break;
                    }
                }, //
                Err(_e) => {
                    write_malformed_response(&mut stream);
                    break;
                }
            },
            Err(_e) => {
                // An error occurred reading from the stream.
                write_malformed_response(&mut stream);
                break;
            }
        }
    }
}

fn validate_request(obj: json::JsonValue) -> Result<json::JsonValue, PrimeTimeError> {
    if obj.has_key("method")
        && obj.has_key("number")
        && !obj["method"].is_null()
        && obj["method"] == "isPrime"
        && !obj["number"].is_null()
    {
        match obj["number"] {
            json::JsonValue::Number(n) => {
                if n.is_nan() {
                    Err(PrimeTimeError::InvalidRequest)
                } else {
                    // We have a valid number. It might be floating point or negative.
                    let (positive, _mantissa, exponent) = n.as_parts();
                    let is_prime = exponent == 0
                        && positive
                        && primal::is_prime(n.as_fixed_point_u64(0).unwrap());

                    let response_obj = object! {method: "isPrime", prime: is_prime};
                    Ok(response_obj)
                }
            }
            _ => Err(PrimeTimeError::InvalidRequest),
        }
    } else {
        Err(PrimeTimeError::InvalidRequest)
    }
}

fn write_malformed_response(stream: &mut TcpStream) {
    stream.write_all(b"ERROR").unwrap();
}
