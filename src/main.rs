use chat::server::HttpServer;

fn main() {
    simple_logger::init().unwrap();

    let server = HttpServer::bind("0.0.0.0", 7878);
    server.start();
}
