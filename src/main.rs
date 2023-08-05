use clap::Parser;
use dependency_graph::{run, Config};

fn main() {
  let config = Config::parse();
  if let Err(e) = run(config) {
    eprintln!("{e}");
    std::process::exit(1);
  }
}
