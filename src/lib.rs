use std::net::{TcpListener, TcpStream};
use threadpool::ThreadPool;

pub fn run_server<F>(port: Option<usize>, num_workers: usize, connection_handler: F)
where
    F: Fn(TcpStream) + Send + Copy + 'static,
{
    let bind_addr = match port {
        Some(p) => format!("0.0.0.0:{}", p),
        None => "0.0.0.0:5001".to_string(),
    };

    let listener = TcpListener::bind(bind_addr).unwrap();
    let pool = ThreadPool::new(num_workers);
    for stream in listener.incoming() {
        pool.execute(move || connection_handler(stream.unwrap()));
    }
}
