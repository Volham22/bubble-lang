use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use linker::LD_PATH;

mod cli;
mod errors;
mod io;
mod linker;
mod objects;

fn main() {
    let cli = cli::Cli::parse();

    match objects::build_objects_targets(
        cli.targets
            .iter()
            .map(|p| p.as_path())
            .collect::<Vec<&Path>>()
            .as_slice(),
        cli.debug,
        cli.emit_llvm,
    ) {
        Ok(objs) => {
            if !cli.compile_only {
                if let Err(e) = linker::link_objects(
                    objs.iter()
                        .map(|p| p.as_path())
                        .collect::<Vec<&Path>>()
                        .as_slice(),
                    &cli.output
                        .unwrap_or(PathBuf::from_str("./program").expect("unreachable")),
                    cli.ld_path
                        .as_ref()
                        .map(|p| p.to_str().expect("failed to convert to path")),
                ) {
                    errors::print_error(
                        &e,
                        &cli.ld_path.unwrap_or(PathBuf::from_str(LD_PATH).unwrap()),
                    )
                    .expect("failed to print error to the terminal");
                    std::process::exit(1);
                }
            }
        }
        Err(ref e) => {
            errors::print_error(
                e,
                &cli.ld_path.unwrap_or(PathBuf::from_str(LD_PATH).unwrap()),
            )
            .expect("failed to print error to the terminal");
            std::process::exit(1);
        }
    }
}
