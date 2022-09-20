use std::io::{Read, Write};
use std::net::TcpStream;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeansToAnEndError {
    #[error("Message type must be 'I' or 'Q'.")]
    InvalidMessageType,

    #[error("If this error is being returned here, there is a major bug.")]
    NotPossible,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Message {
    Insert { timestamp: i32, price: i32 },
    Query { mintime: i32, maxtime: i32 },
}

impl Message {
    fn from_network_bytes(bytes: [u8; 9]) -> Result<Self, MeansToAnEndError> {
        let mut message_type: u8 = 0;
        let mut first_i32: [u8; 4] = [0; 4];
        let mut second_i32: [u8; 4] = [0; 4];

        // Extract bytes to each part of the message.
        for (index, byte) in bytes.iter().enumerate() {
            match index {
                0 => {
                    message_type = *byte;
                }
                (1..=4) => {
                    first_i32[index - 1] = *byte;
                }
                (5..=8) => {
                    second_i32[index - 5] = *byte;
                }
                _ => {
                    return Err(MeansToAnEndError::NotPossible);
                }
            }
        }

        // Network-byte order is big endian. The method `from_be_bytes`
        // interprets bytes as big endian (hence the `be`).
        match message_type as char {
            'I' => Ok(Message::Insert {
                timestamp: i32::from_be_bytes(first_i32),
                price: i32::from_be_bytes(second_i32),
            }),
            'Q' => Ok(Message::Query {
                mintime: i32::from_be_bytes(first_i32),
                maxtime: i32::from_be_bytes(second_i32),
            }),
            _ => Err(MeansToAnEndError::InvalidMessageType),
        }
    }

    pub fn to_network_bytes(&self) -> [u8; 9] {
        match self {
            Message::Insert { timestamp, price } => {
                let mut bytes: [u8; 9] = [0; 9];
                bytes[0] = b'I';
                Message::i32_to_network_bytes(*timestamp, &mut bytes[1..5]);
                Message::i32_to_network_bytes(*price, &mut bytes[5..9]);
                bytes
            }
            Message::Query { mintime, maxtime } => {
                let mut bytes: [u8; 9] = [0; 9];
                bytes[0] = b'Q';
                Message::i32_to_network_bytes(*mintime, &mut bytes[1..5]);
                Message::i32_to_network_bytes(*maxtime, &mut bytes[5..9]);
                bytes
            }
        }
    }

    fn i32_to_network_bytes(an_i32: i32, buf: &mut [u8]) {
        for (i, x) in an_i32.to_be_bytes().iter().enumerate() {
            buf[i] = *x;
        }
    }
}

/*
 * - Each connection is a session.
 * - Sessions will store an AssetPriceDB
 * - AssetPriceDBs will store a vector of AssetPrices
 * - AssertPriceDBs will support two operations: insert, and query.
 * - AssetPrice will be a struct of two ints: timestamp and price, both i32s.
 * - Message will be an enum representing a message we receive.
 *   - Messages can be Insert or Query.
 *   - Insert and query will be structs containing the data each message stores.
 *   - These will be used for validation and convenience.
 * - Given a message on a connection, this process will do something with the AssetPriceDB in the session.
 *
 *
 * TODO:
 * - Write 1-3 basic happy path tests that run the server and send query and insert messages.
 * - Implement message parsing with some unit tests maybe.
 * - Implement the AssetDB with some unit tests.
 * - Glue. Run tests.
 */

struct Session {
    db: AssetPriceDB,
}

impl Session {
    pub fn new() -> Session {
        Session {
            db: AssetPriceDB::new(),
        }
    }
}

// The derivation of *Eq and *Ord below will compare timestamp first
// then price, which is fine. We want this to be ordered by timestamp.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct AssetPrice {
    timestamp: i32,
    price: i32,
}

struct AssetPriceDB {
    asset_prices: Vec<AssetPrice>,
}

impl AssetPriceDB {
    pub fn new() -> AssetPriceDB {
        AssetPriceDB {
            asset_prices: vec![],
        }
    }

    fn insert(&mut self, timestamp: i32, price: i32) {
        self.remove_if_exists(timestamp);
        self.asset_prices.push(AssetPrice { timestamp, price });
    }

    fn query(&self, mintime: i32, maxtime: i32) -> i32 {
        if maxtime < mintime {
            return 0;
        }

        let mut num_assets = 0;
        let mut total_asset_price = 0i64;
        for (_, v) in self.asset_prices.iter().enumerate() {
            if mintime <= v.timestamp && v.timestamp <= maxtime {
                total_asset_price += v.price as i64;
                num_assets += 1;
            }
        }

        if num_assets == 0 {
            0
        } else {
            (total_asset_price / num_assets) as i32
        }
    }

    fn remove_if_exists(&mut self, timestamp: i32) {
        for (i, v) in self.asset_prices.iter().enumerate() {
            if v.timestamp == timestamp {
                self.asset_prices.remove(i);
                break;
            }
        }
    }
}

pub fn handle_connection(mut stream: TcpStream) {
    let mut session = Session::new();

    loop {
        let mut buf: [u8; 9] = [0; 9];
        match stream.read_exact(&mut buf) {
            Ok(_) => {
                let message = Message::from_network_bytes(buf);
                if let Ok(m) = message {
                    handle_message(&mut stream, &mut session, m);
                } // If the message doesn't parse, ignore it.
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Connection's closed.
                break;
            }
            Err(_) => {
                // Error reading from the stream.
                // Let's just close the connection.
                break;
            }
        }
    }
}

fn handle_message(stream: &mut TcpStream, session: &mut Session, message: Message) {
    match message {
        Message::Insert { timestamp, price } => {
            session.db.insert(timestamp, price);
        }
        Message::Query { mintime, maxtime } => {
            let mean = session.db.query(mintime, maxtime);
            stream.write_all(&mean.to_be_bytes()).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::Message;

    #[test]
    fn test_message_network_byte_serde() {
        let message = Message::Insert {
            timestamp: 2394,
            price: 994920,
        };

        // Serialize then deserialize and make sure it still matches.
        assert_eq!(
            message,
            Message::from_network_bytes(message.to_network_bytes()).unwrap()
        );
    }
}
