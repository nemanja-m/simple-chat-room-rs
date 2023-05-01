use chat::server::HttpServer;

fn main() {
    simple_logger::init().unwrap();

    HttpServer::start("0.0.0.0:7878", 8);
}
