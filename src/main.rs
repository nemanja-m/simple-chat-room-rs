use chat::server::HttpServer;

fn main() {
    simple_logger::init().unwrap();

    HttpServer::build().start("0.0.0.0", 7878);
}
