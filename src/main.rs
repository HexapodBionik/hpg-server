use std::env;

use hpg_server::start_server;

fn main() {
    start_server(env::args().skip(1).collect::<Vec<String>>());
}
