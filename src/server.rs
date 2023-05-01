use log::info;

use std::io::Write;
use std::net::{TcpListener, ToSocketAddrs};

use crate::handler::handle_request;
use crate::request::HttpRequest;
use crate::state::{StaticFiles, ThreadSafeChatRoom};
use crate::thread_pool::ThreadPool;

pub struct HttpServer;

impl HttpServer {
    pub fn start<A>(address: A, num_threads: usize)
    where
        A: ToSocketAddrs,
    {
        let listener = TcpListener::bind(address).unwrap();

        info!(
            "Listening at http://{}/ for connections",
            listener.local_addr().unwrap()
        );

        let state = ThreadSafeChatRoom::new();
        let static_files = StaticFiles::new();
        let thread_pool = ThreadPool::new(num_threads);

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            let static_files = static_files.clone();
            let state = state.clone();

            thread_pool.execute(move || {
                let request = HttpRequest::new(&stream, static_files, state);
                let response = handle_request(&request);

                stream.write_all(response.as_bytes()).unwrap();
            });
        }
    }
}
