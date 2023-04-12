extern crate lalrpop;

const LALRPOP_DIR: &str = "src/parse";

fn main() {
    lalrpop::Configuration::default()
        .set_in_dir(LALRPOP_DIR)
        .set_out_dir(LALRPOP_DIR)
        .process_current_dir()
        .unwrap();
}
