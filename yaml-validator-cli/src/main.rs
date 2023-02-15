use clap::Parser;
use yaml_validator_cli::{actual_main, Opt};

fn main() {
    let opt = Opt::parse();

    match actual_main(&opt) {
        Ok(()) => println!("all files validated successfully!"),
        Err(e) => {
            eprint!("{e}");
            std::process::exit(1);
        }
    }
}
