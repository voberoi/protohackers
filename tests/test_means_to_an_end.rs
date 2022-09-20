use protohackers::means_to_an_end::Message;
use std::io::{Read, Write};
use std::net::TcpStream;

mod common;

#[test]
fn test_single_client() {
    let server = common::ServerProcess::run_means_to_an_end();

    let mut stream = server.get_stream();
    insert(&mut stream, 1000, 100);
    insert(&mut stream, 1020, 200);
    insert(&mut stream, 1040, 250);

    // DB contains:
    // 1000, 100
    // 1020, 200
    // 1040, 250

    // Floating point can round in any direction in the protocol, but our
    // implementation will always return the floor.
    let mut mean = query(&mut stream, 999, 1041);
    assert_eq!(mean, 183);

    // Range is inclusive.
    mean = query(&mut stream, 1000, 1040);
    assert_eq!(mean, 183);

    // Some arbitrary larger range.
    mean = query(&mut stream, 500, 20000);
    assert_eq!(mean, 183);

    // A subset of values.
    mean = query(&mut stream, 900, 1030);
    assert_eq!(mean, 150);

    // min greater than max must return 0.
    mean = query(&mut stream, 2000, 1000);
    assert_eq!(mean, 0);

    // A range without a value must return 0
    mean = query(&mut stream, 2000, 8000);
    assert_eq!(mean, 0);

    // min == max, with a value at that point
    mean = query(&mut stream, 1020, 1020);
    assert_eq!(mean, 200);

    // min == max, without a value at that point
    mean = query(&mut stream, 1028, 1028);
    assert_eq!(mean, 0);

    // Insert some more values, including negative timestamps and prices.
    // These should all work. The protocol doesn't define what to do with
    // negative timestamp, so I'll assume this must work. The protocol
    // explicitly mentions that negative prices are possible.

    // Insert some negative timestamp values.
    insert(&mut stream, -1000, 100);
    insert(&mut stream, -1020, 200);
    insert(&mut stream, -1040, 250);

    // Insert some negative price values.
    insert(&mut stream, 1010, -100);
    insert(&mut stream, -2000, -100);

    // Insert some more prices that modify results for queries we've already
    // done.
    insert(&mut stream, 1025, 300);

    // DB contains:
    // -2000, -100
    // -1040, 250
    // -1020, 200
    // -1000, 100
    // 1000, 100
    // 1010, -100
    // 1020, 200
    // 1025, 300
    // 1040, 250

    mean = query(&mut stream, 999, 1041);
    assert_eq!(mean, 150);

    mean = query(&mut stream, 1000, 1040);
    assert_eq!(mean, 150);

    mean = query(&mut stream, 500, 20000);
    assert_eq!(mean, 150);

    mean = query(&mut stream, 900, 1030);
    assert_eq!(mean, 125);

    mean = query(&mut stream, 2000, 1000);
    assert_eq!(mean, 0);

    mean = query(&mut stream, 2000, 8000);
    assert_eq!(mean, 0);

    mean = query(&mut stream, 1020, 1020);
    assert_eq!(mean, 200);

    mean = query(&mut stream, 1025, 1025);
    assert_eq!(mean, 300);

    mean = query(&mut stream, 1028, 1028);
    assert_eq!(mean, 0);

    mean = query(&mut stream, -1050, -999);
    assert_eq!(mean, 183);

    mean = query(&mut stream, -1999, -999);
    assert_eq!(mean, 183);

    mean = query(&mut stream, -2000, -999);
    assert_eq!(mean, 112);

    mean = query(&mut stream, -2001, -999);
    assert_eq!(mean, 112);

    mean = query(&mut stream, -500, 500);
    assert_eq!(mean, 0);

    // Test inserting the same timestamps.
    // The protocol says this is undefined behavior.
    // Our implementation will overwrite existing values.
    insert(&mut stream, 1000, 250);
    insert(&mut stream, -1000, 350);

    // DB contains:
    // -2000, -100
    // -1040, 250
    // -1020, 200
    // -1000, 350
    // 1000, 250
    // 1010, -100
    // 1020, 200
    // 1025, 300
    // 1040, 250

    mean = query(&mut stream, 999, 1041);
    assert_eq!(mean, 180);

    mean = query(&mut stream, -1025, -900);
    assert_eq!(mean, 275);
}

#[test]
fn test_mutiple_clients() {
    let server = common::ServerProcess::run_means_to_an_end();

    let mut stream1 = server.get_stream();
    insert(&mut stream1, 300, 100);
    insert(&mut stream1, 400, 200);
    insert(&mut stream1, 650, 250);

    let mut stream2 = server.get_stream();
    insert(&mut stream2, 1300, 200);
    insert(&mut stream2, 1400, 300);
    insert(&mut stream2, 1650, 350);

    let mut stream3 = server.get_stream();
    insert(&mut stream3, 2300, 300);
    insert(&mut stream3, 2400, 400);
    insert(&mut stream3, 2650, 450);

    let mut mean = query(&mut stream1, 200, 700);
    assert_eq!(mean, 183);

    mean = query(&mut stream1, 200, 500);
    assert_eq!(mean, 150);

    mean = query(&mut stream2, 1200, 1700);
    assert_eq!(mean, 283);

    mean = query(&mut stream2, 1200, 1500);
    assert_eq!(mean, 250);

    mean = query(&mut stream3, 2200, 2700);
    assert_eq!(mean, 383);

    mean = query(&mut stream3, 2200, 2500);
    assert_eq!(mean, 350);

    // Test inserting a new value for the same timestamp on one connection.
    insert(&mut stream1, 300, 300);

    // This mean changes.
    mean = query(&mut stream1, 200, 700);
    assert_eq!(mean, 250);

    // This mean changes.
    mean = query(&mut stream1, 200, 500);
    assert_eq!(mean, 250);

    // The rest of these remain the same.
    mean = query(&mut stream2, 1200, 1700);
    assert_eq!(mean, 283);

    mean = query(&mut stream2, 1200, 1500);
    assert_eq!(mean, 250);

    mean = query(&mut stream3, 2200, 2700);
    assert_eq!(mean, 383);

    mean = query(&mut stream3, 2200, 2500);
    assert_eq!(mean, 350);
}

fn insert(stream: &mut TcpStream, timestamp: i32, price: i32) {
    let message = Message::Insert { timestamp, price };
    stream.write_all(&message.to_network_bytes()).unwrap();
}

fn query(stream: &mut TcpStream, mintime: i32, maxtime: i32) -> i32 {
    let message = Message::Query { mintime, maxtime };
    stream.write_all(&message.to_network_bytes()).unwrap();

    let mut buf: [u8; 4] = [0; 4];
    stream.read_exact(&mut buf).unwrap();

    i32::from_be_bytes(buf)
}
