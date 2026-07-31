use std::{env, fs, path::PathBuf};

use schema_rust::build::{CargoSchemaMetadata, GenerationDriver, GenerationPlan, ModuleEmission};

fn main() {
    SchemaBuild::from_environment().run();
}

struct SchemaBuild {
    crate_root: PathBuf,
}

impl SchemaBuild {
    fn from_environment() -> Self {
        Self {
            crate_root: PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir set")),
        }
    }

    fn run(&self) {
        println!("cargo:rerun-if-changed=schema/assembly.schema");
        println!("cargo:rerun-if-changed=src/schema/assembly.rs");

        let plan = GenerationPlan::new(&self.crate_root, "skills", "0.4.0")
            .with_module(ModuleEmission::declaration_module("assembly"));

        if env::var_os("SKILLS_UPDATE_SCHEMA_ARTIFACTS").is_some() {
            GenerationDriver::new(plan)
                .generate()
                .expect("generate skills schema artifacts")
                .write_or_check("SKILLS_UPDATE_SCHEMA_ARTIFACTS")
                .expect("checked-in skills schema artifacts are fresh");
            self.migrate_schema_artifacts_to_dotos();
        }
        CargoSchemaMetadata::new("skills").emit_schema_directory(&self.crate_root);
    }

    fn migrate_schema_artifacts_to_dotos(&self) {
        let artifact = self.crate_root.join("src/schema/assembly.rs");
        let source = fs::read_to_string(&artifact).expect("read generated schema artifact");
        let dotos = source
            .replace("nota-text", "dotos-text")
            .replace("Nota", "Dotos")
            .replace("nota", "dotos");
        fs::write(artifact, dotos).expect("write DOTOS schema artifact");
    }
}
