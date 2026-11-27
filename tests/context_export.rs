use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn roots(base: &Path) -> [PathBuf; 3] {
    ["first", "second", "third"].map(|name| {
        let root = base.join(name);
        fs::create_dir(&root).unwrap();
        root
    })
}

fn fixture(base: &Path) -> ([PathBuf; 3], [String; 3]) {
    let roots = roots(base);
    let revisions = ["a".repeat(40), "a".repeat(40), "c".repeat(40)];
    write(
        roots[0].join("modules.datom"),
        "[ { { Vision kept Single.0 } «vision/kept/0.md» } ]\n",
    );
    write(roots[0].join("vision/kept/0.md"), "exact first bytes\n");
    write(roots[0].join("vision/extra/0.md"), "undeclared first\n");
    write(
        roots[1].join("modules.datom"),
        "[ { { Notion held Single.1 } «notion/held/1.md» } ]\n",
    );
    write(roots[1].join("notion/held/1.md"), "exact second bytes\n");
    write(roots[1].join("notion/extra/0.md"), "undeclared second\n");
    write(
        roots[1].join("vision/kept/0.md"),
        "undeclared same-revision same-path decoy\n",
    );
    write(
        roots[2].join("modules.datom"),
        "[ { { State hosts Single.0 } «state/hosts/0.md» } ]\n",
    );
    write(roots[2].join("state/hosts/0.md"), "exact third bytes\n");
    write(roots[2].join("state/extra/0.md"), "undeclared third\n");
    (roots, revisions)
}

fn invoke(
    output: &Path,
    names: [&str; 3],
    roots: &[PathBuf; 3],
    revisions: &[String; 3],
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_curriculum-context-export"));
    command.arg(output);
    for index in 0..3 {
        command
            .arg(names[index])
            .arg(&roots[index])
            .arg(&revisions[index]);
    }
    command.output().unwrap()
}

fn files(root: &Path) -> Vec<String> {
    fn visit(root: &Path, directory: &Path, output: &mut Vec<String>) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, output);
            } else {
                output.push(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    let mut output = Vec::new();
    visit(root, root, &mut output);
    output.sort();
    output
}

#[test]
fn exports_only_typed_admitted_files_with_existing_inventory_shape() {
    let temporary = tempfile::tempdir().unwrap();
    let (roots, revisions) = fixture(temporary.path());
    let output = temporary.path().join("export");
    let result = invoke(
        &output,
        ["psyche-pattern", "mind-pattern", "field-pattern"],
        &roots,
        &revisions,
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        files(&output),
        [
            "field-pattern/state/hosts/0.md",
            "mind-pattern/notion/held/1.md",
            "modules",
            "psyche-pattern/vision/kept/0.md",
        ]
    );
    assert_eq!(
        fs::read(output.join("psyche-pattern/vision/kept/0.md")).unwrap(),
        b"exact first bytes\n"
    );
    assert_eq!(
        fs::read_to_string(output.join("modules")).unwrap(),
        format!(
            "notion:held:1 {} notion/held/1.md\nstate:hosts:0 {} state/hosts/0.md\nvision:kept:0 {} vision/kept/0.md\n",
            revisions[1], revisions[2], revisions[0]
        )
    );
    assert!(!output.join("psyche-pattern/vision/extra/0.md").exists());
    assert!(!output.join("mind-pattern/notion/extra/0.md").exists());
    assert!(!output.join("mind-pattern/vision/kept/0.md").exists());
    assert!(!output.join("field-pattern/state/extra/0.md").exists());

    let repeated = invoke(
        &output,
        ["psyche-pattern", "mind-pattern", "field-pattern"],
        &roots,
        &revisions,
    );
    assert!(!repeated.status.success());
    assert_eq!(files(&output).len(), 4);
}

#[test]
fn refuses_invalid_names_overlaps_unadmitted_declarations_and_ambiguous_paths_without_output() {
    let temporary = tempfile::tempdir().unwrap();
    let (roots, revisions) = fixture(temporary.path());

    let invalid = temporary.path().join("invalid");
    assert!(
        !invoke(
            &invalid,
            ["Bad", "mind-pattern", "field-pattern"],
            &roots,
            &revisions
        )
        .status
        .success()
    );
    assert!(!invalid.exists());

    let overlap = temporary.path().join("overlap");
    let overlapping = [roots[0].clone(), roots[0].clone(), roots[2].clone()];
    assert!(
        !invoke(&overlap, ["one", "two", "three"], &overlapping, &revisions)
            .status
            .success()
    );
    assert!(!overlap.exists());

    write(
        roots[1].join("modules.datom"),
        "[ { { Notion other Single.0 } «vision/kept/0.md» } ]\n",
    );
    let ambiguous = temporary.path().join("ambiguous");
    assert!(
        !invoke(&ambiguous, ["one", "two", "three"], &roots, &revisions)
            .status
            .success()
    );
    assert!(!ambiguous.exists());
    write(
        roots[1].join("modules.datom"),
        "[ { { Notion held Single.1 } «notion/held/1.md» } ]\n",
    );

    fs::remove_file(roots[2].join("state/hosts/0.md")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("../extra/0.md", roots[2].join("state/hosts/0.md")).unwrap();
    let symlinked = temporary.path().join("symlinked");
    assert!(
        !invoke(&symlinked, ["one", "two", "three"], &roots, &revisions)
            .status
            .success()
    );
    assert!(!symlinked.exists());

    fs::remove_file(roots[2].join("state/hosts/0.md")).unwrap();
    write(roots[2].join("state/hosts/0.md"), "exact third bytes\n");
    write(
        roots[0].join("modules.datom"),
        "[ { { Vision kept Single.0 } «vision/with space/0.md» } ]\n",
    );
    write(
        roots[0].join("vision/with space/0.md"),
        "ambiguous inventory path\n",
    );
    let whitespace = temporary.path().join("whitespace");
    assert!(
        !invoke(&whitespace, ["one", "two", "three"], &roots, &revisions)
            .status
            .success()
    );
    assert!(!whitespace.exists());
    assert!(fs::read_dir(temporary.path()).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".curriculum-context-export-")
    }));
}
