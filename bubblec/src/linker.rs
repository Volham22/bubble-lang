use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::errors::{CompilerError, CompilerResult};

const LD_LOADER_PATH: &str = "/lib64/ld-linux-x86-64.so.2";
const LD_PATH: &str = "/usr/bin/ld";

pub fn link_objects(
    objects: &[&Path],
    executable_path: &Path,
    linker_path: Option<&str>,
) -> CompilerResult<()> {
    let linker_child = Command::new(linker_path.unwrap_or(LD_PATH))
        .arg("-m")
        .arg("elf_x86_64")
        .arg("/usr/lib64/crt1.o") // C runtime
        .arg("/usr/lib64/crti.o") // C runtime
        .arg("/usr/lib64/crtn.o") // C runtime
        .arg("-lc") // Link Lib C
        .arg("-dynamic-linker")
        .arg(LD_LOADER_PATH)
        .arg("-o")
        .arg(executable_path)
        .args(objects)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn ld");

    let link_output = linker_child
        .wait_with_output()
        .expect("Failed to wait linker");

    if !link_output.status.success() {
        Err(CompilerError::Linker(
            String::from_utf8_lossy(&link_output.stderr).to_string(),
        ))
    } else {
        Ok(())
    }
}
