mod support;
use std::{
    fs,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
struct Running(Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn invoke(runtime: &std::path::Path, text: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_curriculum"))
        .env("XDG_RUNTIME_DIR", runtime)
        .arg(text)
        .output()
        .unwrap()
}
#[test]
fn native_nexus_composes_by_ids_and_refuses_retired_paths_and_inline_prose() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path();
    let runtime = base.join("runtime");
    fs::create_dir(&runtime).unwrap();
    let records = base.join("flows");
    fs::create_dir(&records).unwrap();
    // The Nexus binds inside an already provisioned runtime directory.
    fs::create_dir(runtime.join("curriculum")).unwrap();
    let stdout_path = base.join("nexus.stdout");
    let stderr_path = base.join("nexus.stderr");
    let roots = [
        base.join("psyche-belief"),
        base.join("mind-belief"),
        base.join("field-belief"),
    ];
    for (index, root) in roots.iter().enumerate() {
        fs::create_dir(root).unwrap();
        fs::create_dir_all(root.join(".git")).unwrap();
        let revision = ["a", "b", "c"][index].repeat(40);
        write_source(root.join(".git/HEAD"), format!("{revision}\n")).unwrap();
    }
    fs::create_dir(roots[1].join("role")).unwrap();
    write_source(
        roots[1].join("role/book-description/0.md"),
        "Publish a current book.",
    )
    .unwrap();
    let refresh_program = base.join("refresh-contexts");
    write_source(&refresh_program, "#!/bin/sh\nexit 0\n").unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&refresh_program, fs::Permissions::from_mode(0o700)).unwrap();
    for kind in ["spirit", "intent"] {
        fs::create_dir(roots[0].join(kind)).unwrap();
    }
    fs::create_dir(roots[2].join("compensation")).unwrap();
    write_source(roots[0].join("spirit/core/0.md"), "SPIRIT_BODY").unwrap();
    write_source(roots[0].join("intent/seat/0.md"), "INTENT_BODY").unwrap();
    write_source(
        roots[2].join("compensation/illustration/0.md"),
        "---\ndescription: instruction\ndependencies: []\nplacement: user\n---\nEXACT_USER_BODY\n{\"message\":{\"usage\":{\"input_tokens\":2}}}\n{% include \"panel:native:0\" %}\n{% include \"panel:missing:0\" %}\n",
    )
    .unwrap();
    write_source(roots[1].join("panel/native/0.md"), "NATIVE_PANEL").unwrap();
    let roles = base.join("roles.datom");
    write_source(
        &roles,
        "Roles.{ [ { book ClaudeAgent sonnet Medium Unrestricted { Role book-description Single.0 } [ { Spirit core Single.0 } { Intent seat Single.0 } { Compensation illustration Single.0 } ] Explicit.[ Read Bash Agent Artifact ArtifactComments ] Any } ] }",
    )
    .unwrap();
    for root in &roots {
        support::declare(root);
    }
    let current_psyche_revision = "a".repeat(40);
    let child = Command::new(env!("CARGO_BIN_EXE_curriculum-nexus"))
        .env("XDG_RUNTIME_DIR", &runtime)
        .env("CURRICULUM_CONTEXT_LAYOUT", "2")
        .env("CURRICULUM_CONTEXT_REFRESH_PROGRAM", &refresh_program)
        .env("CURRICULUM_ROLES_FILE", &roles)
        .env("CURRICULUM_PSYCHES_REPOSITORY_DIR", &roots[0])
        .env("CURRICULUM_MIND_CONTEXT_DIR", &roots[1])
        .env("CURRICULUM_FIELD_CONTEXT_DIR", &roots[2])
        .env("CURRICULUM_RECORDS_DIR", &records)
        .stdout(Stdio::from(fs::File::create(&stdout_path).unwrap()))
        .stderr(Stdio::from(fs::File::create(&stderr_path).unwrap()))
        .spawn()
        .unwrap();
    let socket = runtime.join("curriculum/curriculum.sock");
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut running = Running(child);
    while !socket.exists() {
        let status = running.0.try_wait().expect("read Nexus child status");
        if status.is_some() || Instant::now() >= deadline {
            let timed_out = status.is_none();
            let terminal = if let Some(status) = status {
                status
            } else {
                running.0.kill().expect("stop timed-out Nexus child");
                running.0.wait().expect("reap timed-out Nexus child")
            };
            panic!(
                "Nexus failed before socket {}; timed_out={timed_out}; status={terminal}; stdout:\n{}\nstderr:\n{}",
                socket.display(),
                fs::read_to_string(&stdout_path).expect("read Nexus stdout"),
                fs::read_to_string(&stderr_path).expect("read Nexus stderr")
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let response = invoke(
        &runtime,
        "ComposeStartup.{ { Field Publisher Quaternary } Claude [ { Spirit core Single.0 } { Intent seat Single.0 } ] [ { Compensation illustration Single.0 } ] [] }",
    );
    assert!(
        response.status.success(),
        "{}",
        String::from_utf8_lossy(&response.stderr)
    );
    let text = String::from_utf8(response.stdout).unwrap();
    assert!(text.starts_with("StartupComposed."));
    assert!(text.contains("EXACT_USER_BODY"));
    assert!(text.contains(r#"{"message":{"usage":{"input_tokens":2}}}"#));
    assert!(text.contains("field-belief"));
    use curriculum::registry::ModuleIdentityConversion;
    use signal_curriculum_nexus::{
        CallerMetaflow, Input, Output, StartupSelection, SystemModules, Target, UserModules,
    };
    let input = Input::ComposeStartup(StartupSelection {
        caller_metaflow: CallerMetaflow {
            first_string: "Field".into(),
            second_string: "Publisher".into(),
            third_string: "Quaternary".into(),
        },
        target: Target::Claude,
        system_modules: SystemModules(vec![
            "spirit:core:0".module_identity().unwrap(),
            "intent:seat:0".module_identity().unwrap(),
        ]),
        user_modules: UserModules(vec![
            "compensation:illustration:0".module_identity().unwrap(),
        ]),
        flow_id_vector: vec![],
    });
    let Output::StartupComposed(typed) = signal_reply(&socket, &input) else {
        panic!("composition must continue despite a missing panel")
    };
    assert_eq!(
        typed.missing_panels.0,
        ["panel:missing:0".module_identity().unwrap()]
    );
    assert_eq!(typed.merged_panels.0.len(), 1);
    let panel = &typed.merged_panels.0[0];
    assert_eq!(
        panel.module_identity,
        "panel:native:0".module_identity().unwrap()
    );
    assert_eq!(panel.source_relative_path.0, "panel/native/0.md");
    assert_eq!(typed.user_text.0.matches("NATIVE_PANEL").count(), 1);
    assert!(!typed.user_text.0.contains("<!-- module: panel:"));
    let sources = invoke(
        &runtime,
        "ResolveSources.[ { Compensation illustration Single.0 } ]",
    );
    assert!(sources.status.success());
    let source_text = String::from_utf8_lossy(&sources.stdout);
    assert!(source_text.contains("compensation/illustration/0.md"));
    let psyche_sources = invoke(&runtime, "ResolveSources.[ { Intent seat Single.0 } ]");
    assert!(psyche_sources.status.success());
    assert!(String::from_utf8_lossy(&psyche_sources.stdout).contains(&current_psyche_revision));
    let role = invoke(&runtime, "ComposeRole.{ book ClaudeAgent }");
    assert!(role.status.success());
    assert!(String::from_utf8_lossy(&role.stdout).contains("EXACT_USER_BODY"));
    assert!(
        String::from_utf8_lossy(&role.stdout)
            .contains(r#"{"message":{"usage":{"input_tokens":2}}}"#)
    );
    for old in [
        "RebuildSkills",
        "CheckSkills",
        "EditSkills.{ [] [] [] }",
        "ComposeStartup.{ { Field Publisher Quaternary } Claude [ { Spirit core Single.0 } { Intent seat Single.0 } ] [ «/tmp/arbitrary.md» ] [] }",
        "ComposeStartup.{ { Field Publisher Quaternary } Claude [ { Spirit core Single.0 } { Intent seat Single.0 } ] [ «inline prompt text» ] [] }",
    ] {
        assert!(!invoke(&runtime, old).status.success(), "accepted {old}");
    }
    assert!(!base.join("tools").exists());
    assert!(!base.join(".claude").exists());
    write_source(
        roots[2].join("compensation/illustration/0.md"),
        "CHANGED_SOURCE",
    )
    .unwrap();
    let changed = invoke(
        &runtime,
        "ComposeStartup.{ { Field Publisher Quaternary } Claude [ { Spirit core Single.0 } { Intent seat Single.0 } ] [ { Compensation illustration Single.0 } ] [] }",
    );
    assert!(
        changed.status.success(),
        "{}",
        String::from_utf8_lossy(&changed.stderr)
    );
    let changed_text = String::from_utf8(changed.stdout).unwrap();
    assert!(changed_text.starts_with("StartupComposed."));
    assert!(changed_text.contains("CHANGED_SOURCE"));
    assert!(!changed_text.contains("EXACT_USER_BODY"));
    let changed_role = invoke(&runtime, "ComposeRole.{ book ClaudeAgent }");
    assert!(changed_role.status.success());
    let changed_role_text = String::from_utf8(changed_role.stdout).unwrap();
    assert!(changed_role_text.contains("CHANGED_SOURCE"));
    assert!(!changed_role_text.contains("EXACT_USER_BODY"));
    write_source(
        roots[2].join("compensation/illustration/0.md"),
        "{{ missing_variable }}",
    )
    .unwrap();
    for request in [
        "ComposeStartup.{ { Field Publisher Quaternary } Claude [ { Spirit core Single.0 } { Intent seat Single.0 } ] [ { Compensation illustration Single.0 } ] [] }",
        "ComposeRole.{ book ClaudeAgent }",
    ] {
        let refused = invoke(&runtime, request);
        assert!(!refused.status.success());
        let diagnostic = String::from_utf8(refused.stderr).unwrap();
        assert!(
            diagnostic.contains("unresolved context variable"),
            "{diagnostic}"
        );
        // The default CLI presents a refusal on stderr. Read the wire itself
        // to assert its typed shape independently of that presentation.
        use datom_codec::{Actualizing, Budget, Potential};
        use protos::ReaderBudget;
        let input = Potential::<signal_curriculum_nexus::Input>::from(request)
            .actualize(&mut Budget {
                remaining: 1_048_576,
                reader: ReaderBudget::try_from(1_048_576).unwrap(),
                depth: 0,
                maximum_depth: 1024,
            })
            .unwrap();
        let output = signal_reply(&socket, &input);
        assert!(
            matches!(output, signal_curriculum_nexus::Output::Rejected(reason) if reason.contains("unresolved context variable"))
        );
    }
}

fn write_source(
    path: impl AsRef<std::path::Path>,
    contents: impl AsRef<[u8]>,
) -> std::io::Result<()> {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, contents)
}

fn signal_reply(
    socket: &std::path::Path,
    input: &signal_curriculum_nexus::Input,
) -> signal_curriculum_nexus::Output {
    use std::io::{Read, Write};
    let mut stream = std::os::unix::net::UnixStream::connect(socket).unwrap();
    let request = rkyv::to_bytes::<rkyv::rancor::Error>(input).unwrap();
    stream
        .write_all(&u32::try_from(request.len()).unwrap().to_be_bytes())
        .unwrap();
    stream.write_all(&request).unwrap();
    let mut length = [0; 4];
    stream.read_exact(&mut length).unwrap();
    let mut reply = vec![0; u32::from_be_bytes(length) as usize];
    stream.read_exact(&mut reply).unwrap();
    rkyv::from_bytes::<signal_curriculum_nexus::Output, rkyv::rancor::Error>(&reply).unwrap()
}
