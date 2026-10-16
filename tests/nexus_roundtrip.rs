use std::{fs,process::{Child,Command,Stdio},time::{Duration,Instant}};
struct Running(Child);
impl Drop for Running {fn drop(&mut self){let _=self.0.kill();let _=self.0.wait();}}
fn invoke(runtime:&std::path::Path,text:&str)->std::process::Output {Command::new(env!("CARGO_BIN_EXE_curriculum")).env("XDG_RUNTIME_DIR",runtime).arg(text).output().unwrap()}
#[test]
fn native_nexus_composes_by_ids_and_refuses_retired_paths_and_inline_prose() {
    let temp=tempfile::tempdir().unwrap();let base=temp.path();let runtime=base.join("runtime");fs::create_dir(&runtime).unwrap();
    // The pinned Nexus entry point binds inside an already provisioned runtime directory.
    fs::create_dir(runtime.join("curriculum")).unwrap();
    let stdout_path=base.join("nexus.stdout");
    let stderr_path=base.join("nexus.stderr");
    let roots=[base.join("psyche-context"),base.join("mind-context"),base.join("field-context")];for root in &roots{fs::create_dir(root).unwrap();}
    for kind in ["spirit","intent"]{fs::create_dir(roots[0].join(kind)).unwrap();}
    fs::create_dir(roots[2].join("compensation")).unwrap();
    fs::write(roots[0].join("spirit/core.md"),"SPIRIT_BODY").unwrap();fs::write(roots[0].join("intent/seat.md"),"INTENT_BODY").unwrap();
    fs::write(roots[2].join("compensation/illustration.md"),"---\ndescription: instruction\ndependencies: []\nplacement: user\n---\nEXACT_USER_BODY").unwrap();
    let child=Command::new(env!("CARGO_BIN_EXE_curriculum-nexus")).env("XDG_RUNTIME_DIR",&runtime).env("CURRICULUM_CONTEXT_LAYOUT","2").env_remove("CURRICULUM_ROLES_FILE")
        .env("CURRICULUM_PSYCHES_REPOSITORY_DIR",&roots[0]).env("CURRICULUM_MIND_CONTEXT_DIR",&roots[1]).env("CURRICULUM_FIELD_CONTEXT_DIR",&roots[2])
        .env("CURRICULUM_PSYCHES_REVISION","a".repeat(40)).env("CURRICULUM_MIND_REVISION","b".repeat(40)).env("CURRICULUM_FIELD_REVISION","c".repeat(40)).stdout(Stdio::from(fs::File::create(&stdout_path).unwrap())).stderr(Stdio::from(fs::File::create(&stderr_path).unwrap())).spawn().unwrap();
    let socket=runtime.join("curriculum/curriculum.sock");let deadline=Instant::now()+Duration::from_secs(10);
    let mut running=Running(child);
    while !socket.exists() {
        let status=running.0.try_wait().expect("read Nexus child status");
        if status.is_some() || Instant::now()>=deadline {
            let timed_out=status.is_none();
            let terminal=if let Some(status)=status {status} else {
                running.0.kill().expect("stop timed-out Nexus child");
                running.0.wait().expect("reap timed-out Nexus child")
            };
            panic!("Nexus failed before socket {}; timed_out={timed_out}; status={terminal}; stdout:\n{}\nstderr:\n{}",
                socket.display(),
                fs::read_to_string(&stdout_path).expect("read Nexus stdout"),
                fs::read_to_string(&stderr_path).expect("read Nexus stderr"));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let response=invoke(&runtime,"ComposeStartup.{ { Field Publisher Quaternary } Claude [ spirit-core intent-seat ] [ compensation-illustration ] }");
    assert!(response.status.success(),"{}",String::from_utf8_lossy(&response.stderr));let text=String::from_utf8(response.stdout).unwrap();assert!(text.starts_with("StartupComposed."));assert!(text.contains("EXACT_USER_BODY"));assert!(text.contains("field-context"));
    let sources=invoke(&runtime,"ResolveSources.[ compensation-illustration ]");assert!(sources.status.success());assert!(String::from_utf8_lossy(&sources.stdout).contains("compensation/illustration.md"));
    for old in ["RebuildSkills","CheckSkills","EditSkills.{ [] [] [] }","ComposeStartup.{ { Field Publisher Quaternary } Claude [ spirit-core intent-seat ] [ «/tmp/arbitrary.md» ] }","ComposeStartup.{ { Field Publisher Quaternary } Claude [ spirit-core intent-seat ] [ «inline prompt text» ] }"]{assert!(!invoke(&runtime,old).status.success(),"accepted {old}");}
    assert!(!base.join("tools").exists());assert!(!base.join(".claude").exists());
    fs::write(roots[2].join("compensation/illustration.md"),"CHANGED_SOURCE").unwrap();
    assert!(!invoke(&runtime,"ComposeStartup.{ { Field Publisher Quaternary } Claude [ spirit-core intent-seat ] [ compensation-illustration ] }").status.success());
}
