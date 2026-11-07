//! `src/generated.rs` is exactly what the declared generator, the dev dependency ethos-zero, makes
//! of the authored `curriculum.ethos`. The committed file is not trusted: it is regenerated here and
//! compared byte for byte, so a stale or hand-edited generated file fails the test.

use ethos_zero::{Actualizing, File, Generating, Potential};

#[test]
fn the_committed_generated_file_is_the_generators_output_for_the_authored_ethos() {
    let root = env!("CARGO_MANIFEST_DIR");
    let source = std::fs::read_to_string(format!("{root}/curriculum.ethos")).expect("read curriculum.ethos");
    let file = Potential::<File>::from(source)
        .actualize()
        .unwrap_or_else(|_| panic!("read curriculum.ethos"));
    let generated = file
        .generate()
        .unwrap_or_else(|_| panic!("generate curriculum.ethos"));
    let committed = std::fs::read_to_string(format!("{root}/src/generated.rs")).expect("read src/generated.rs");
    assert_eq!(generated, committed, "src/generated.rs is stale: regenerate it from curriculum.ethos");
}
