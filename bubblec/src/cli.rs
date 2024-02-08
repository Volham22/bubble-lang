use std::path::PathBuf;

use clap_derive::Parser;

#[derive(Parser)]
#[command(version, about)]
#[command(name = "bubblec")]
/// Compiler for the bubble language
pub struct Cli {
    /// Show debug infos (for compiler debugging purposes)
    #[arg(short, long)]
    pub debug: bool,
    /// Do not link the program (only produces object files)
    #[arg(short, long)]
    pub compile_only: bool,
    /// Emit llvm IR code
    #[arg(short, long)]
    pub emit_llvm: bool,
    /// Set an alternative ld path (the linker must support ld style arguments)
    #[arg(short, long)]
    pub ld_path: Option<PathBuf>,
    /// Executable output path
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Targets to compile or link
    pub targets: Vec<PathBuf>,
}
