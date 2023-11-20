use args::Args;
use getopts::Occur;
use std::env;
use std::process;

use hpg_server::start_server;

fn main() {
    let mut args = Args::new("hpg-server", "Hexapod PWM Gadget USB device server");
    args.option(
        "e",
        "endpoints",
        "The number of endpoints to configure",
        "NUMBER",
        Occur::Req,
        None,
    );

    match args.parse(env::args().collect::<Vec<String>>()) {
        Ok(()) => {}
        Err(_) => {
            println!("Usage: hpg-server -e <eps_cnt>");
            process::exit(0);
        }
    };

    let eps_cnt = match args.value_of("endpoints") {
        Ok(eps_cnt) => eps_cnt,
        Err(_) => panic!(),
    };
    start_server(eps_cnt);
}
