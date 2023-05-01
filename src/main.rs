use chat::server::HttpServer;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    #[arg(long, default_value = "7878")]
    port: usize,

    #[arg(long, default_value = "4")]
    threads: usize,
}

fn main() {
    simple_logger::init().unwrap();

    let args = Args::parse();
    let address = format!("{}:{}", args.host, args.port);

    HttpServer::start(address, args.threads);
}
