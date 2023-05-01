use log::info;
use std::collections::HashMap;

use std::io::Write;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::Mutex;

use crate::data::Data;
use crate::handler::handle_request;
use crate::request::HttpRequest;
use crate::state::ChatRoom;
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

        let thread_pool = ThreadPool::new(num_threads);

        let static_files = Data::new(load_static_files());
        let state = Data::new(Mutex::new(ChatRoom::new()));

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

fn load_static_files() -> HashMap<String, String> {
    let mut map = HashMap::new();

    let paths = std::fs::read_dir("static/").unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let index = std::fs::read_to_string(&path).unwrap();
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        map.insert(filename, index);
    }

    map
}
