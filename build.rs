//! Curriculum's own file formats, its Memory and its Emission are generated
//! from curriculum.ethos, memory.ethos and emission.ethos by ethos-zero; the
//! committed Rust must equal fresh generation.

use ethos_zero::{Actualizing, File, Generating, Potential};

fn main() {
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest"));
    for (source_path, generated_path) in [
        ("curriculum.ethos", "src/generated.rs"),
        ("memory.ethos", "src/memory.rs"),
        ("emission.ethos", "src/emission.rs"),
    ] {
        println!("cargo:rerun-if-changed={source_path}");
        println!("cargo:rerun-if-changed={generated_path}");
        let source = std::fs::read_to_string(root.join(source_path)).expect("source");
        let file = Potential::<File>::from(source)
            .actualize()
            .unwrap_or_else(|_| panic!("read {source_path}"));
        let generated = file
            .generate()
            .unwrap_or_else(|_| panic!("generate {source_path}"));
        assert_eq!(
            generated,
            std::fs::read_to_string(root.join(generated_path)).expect("generated"),
            "{generated_path} is stale; regenerate it from {source_path}"
        );
    }
}
