use std::{io, path::PathBuf, process::ExitCode};

use clap::Parser;
use rtree::{Cli, Tree, TreeOptions};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let options: TreeOptions = cli.into();

    match Tree::build(PathBuf::from(&options.path), options.clone()) {
        Ok(tree) => {
            let stdout = io::stdout();
            if let Err(error) = tree.write_to(&mut stdout.lock(), &options) {
                eprintln!("rtree: failed to write output: {error}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("rtree: {error}");
            ExitCode::FAILURE
        }
    }
}
