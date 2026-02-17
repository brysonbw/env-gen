use std::process;

use chroma_print::print_error;
use env_gen::Config;

fn main() {
    if let Err(error) = Config::run() {
        print_error!("\nError: {}", error);
        process::exit(1);
    }
}
