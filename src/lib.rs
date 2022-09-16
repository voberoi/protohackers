use std::net::{TcpListener, TcpStream};
use threadpool::ThreadPool;

pub fn run_server<F>(num_workers: usize, connection_handler: F)
where
    F: Fn(TcpStream) + Send + Copy + 'static,
{
    let listener = TcpListener::bind("0.0.0.0:5001").unwrap();
    let pool = ThreadPool::new(num_workers);
    for stream in listener.incoming() {
        pool.execute(move || connection_handler(stream.unwrap()));
    }
}
