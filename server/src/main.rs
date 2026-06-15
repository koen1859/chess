mod server;

use clap::Parser;
use std::path::PathBuf;
use server::server::serve;

#[derive(Parser)]
#[command(name = "chess-server", about = "Serves the Rust Chess WASM app")]
struct Args {
    #[arg(short, long, default_value = "8080", env = "PORT")]
    port: u16,

    #[arg(short, long, default_value = "dist")]
    dir: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    serve(args.port, args.dir).await;
}
