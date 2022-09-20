use json::object;

mod common;

#[test]
fn test_with_prime_integer() {
    let server = common::ServerProcess::run_prime_time();
    let mut stream = server.get_stream();

    let request_body = object! {
        method: "isPrime", number: 97
    };

    common::write_json_line(&mut stream, &request_body);
    let response = common::read_line(&mut stream);
    let response = json::parse(&response).unwrap();

    assert_eq!(response, object! {method: "isPrime", prime: true});
    assert!(common::connection_is_open(&stream));
}

#[test]
fn test_with_non_prime_integer() {
    let server = common::ServerProcess::run_prime_time();

    let mut stream = server.get_stream();

    let request_body = object! {
        method: "isPrime", number: 98
    };

    common::write_json_line(&mut stream, &request_body);
    let response = common::read_line(&mut stream);
    let response = json::parse(&response).unwrap();

    assert_eq!(response, object! {method: "isPrime", prime: false});
    assert!(common::connection_is_open(&stream));
}

#[test]
fn test_with_negative_integer() {
    let server = common::ServerProcess::run_prime_time();
    let mut stream = server.get_stream();

    let request_body = object! {
        method: "isPrime", number: -47
    };

    common::write_json_line(&mut stream, &request_body);
    let response = common::read_line(&mut stream);
    let response = json::parse(&response).unwrap();

    assert_eq!(response, object! {method: "isPrime", prime: false});
    assert!(common::connection_is_open(&stream));
}

#[test]
fn test_with_floating_point_number() {
    let server = common::ServerProcess::run_prime_time();
    let mut stream = server.get_stream();

    let request_body = object! {
        method: "isPrime", number: 4.24
    };

    common::write_json_line(&mut stream, &request_body);
    let response = common::read_line(&mut stream);
    let response = json::parse(&response).unwrap();

    assert_eq!(response, object! {method: "isPrime", prime: false});
    assert!(common::connection_is_open(&stream));
}

#[test]
fn test_with_malformed_json() {
    let server = common::ServerProcess::run_prime_time();
    let mut stream = server.get_stream();

    let request_body = "{ method \"isPrime\", number: 97}"; // Missing a colon
    common::write_line(&mut stream, request_body.to_string());
    let response = common::read_line(&mut stream);

    assert_eq!(response, "ERROR");
    assert!(!common::connection_is_open(&stream));
}

#[test]
fn test_with_missing_field() {
    let server = common::ServerProcess::run_prime_time();
    let mut stream = server.get_stream();

    let request_body = object! {
        method: "isPrime"
    };
    common::write_line(&mut stream, request_body.to_string());
    let response = common::read_line(&mut stream);

    assert_eq!(response, "ERROR");
    assert!(!common::connection_is_open(&stream));
}

#[test]
fn test_with_incorrect_method() {
    let server = common::ServerProcess::run_prime_time();
    let mut stream = server.get_stream();

    let request_body = object! {
        method: "isNotPrime",
        number: 97
    };

    common::write_line(&mut stream, request_body.to_string());
    let response = common::read_line(&mut stream);

    assert_eq!(response, "ERROR");
    assert!(!common::connection_is_open(&stream));
}
