mod client;
mod player;
mod camera;
use std::env;
mod ground;
mod water;
fn main() {
    let mut args = env::args();
    args.next();
    match args.next().as_ref().map(|s| s.as_str()) {
        Some("client") => {
            println!("Running on client mode");
            client::run();
        }
        _ => {
            println!("Usage : {} [client]", args.next().unwrap());
        }
    }
}