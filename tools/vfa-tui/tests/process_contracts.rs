//! Executable preparation contracts; not D7/native descendant or stream acceptance.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use vfa_tui::subprocess::program::{ProgramError, ProgramSpec};
use vfa_tui::subprocess::{SubprocessExecutor, SubprocessHandle};

struct Helper {
    _temp: TempDir,
    executable: PathBuf,
    cwd: PathBuf,
}

impl Helper {
    fn compile() -> Self {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let artifacts = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| manifest.join("target"))
            .join("process-contract-fixtures");
        std::fs::create_dir_all(&artifacts).unwrap();
        let temp = tempfile::tempdir_in(artifacts).unwrap();
        let root = temp.path().canonicalize().unwrap();
        let directory = root.join("helper Ω 空 spaces");
        std::fs::create_dir(&directory).unwrap();
        let executable =
            directory.join(format!("controlled-helper{}", std::env::consts::EXE_SUFFIX));
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
        let output = Command::new(compiler)
            .current_dir(manifest)
            .args(["--edition=2021", "--crate-name=vfa_process_contract_helper"])
            .arg(manifest.join("tests/support/process_helper.rs"))
            .arg("-o")
            .arg(&executable)
            .output()
            .expect("an installed compiler is required; never skip or install a helper compiler");
        assert!(
            output.status.success(),
            "helper compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let cwd = root.join("selected cwd Ω 空 ; literal");
        std::fs::create_dir(&cwd).unwrap();
        Self {
            _temp: temp,
            executable,
            cwd,
        }
    }

    fn spec(
        &self,
        arguments: Vec<OsString>,
        environment: Vec<(OsString, OsString)>,
    ) -> ProgramSpec {
        // Explicit Windows loader context only; never ambient inheritance in
        // ProgramSpec. Other caller-selected keys remain absent.
        #[cfg(windows)]
        let environment = {
            let mut environment = environment;
            if let Some(root) = std::env::var_os("SystemRoot") {
                environment.push((OsString::from("SystemRoot"), root));
            }
            environment
        };
        ProgramSpec::resolve(&self.executable, arguments, &self.cwd, environment).unwrap()
    }
}

async fn lines(
    handle: &mut SubprocessHandle,
    stdout_count: usize,
    stderr_count: usize,
) -> (Vec<String>, Vec<String>) {
    tokio::time::timeout(Duration::from_secs(10), async {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        loop {
            while let Some(line) = handle.try_recv_stdout() {
                stdout.push(line.content);
            }
            while let Some(line) = handle.try_recv_stderr() {
                stderr.push(line.content);
            }
            if stdout.len() >= stdout_count && stderr.len() >= stderr_count {
                return (stdout, stderr);
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("finite helper output must actually arrive; no sleep-as-proof")
}

#[tokio::test]
async fn typed_launch_preserves_literal_native_argv_cwd_and_explicit_environment() {
    let helper = Helper::compile();
    let marker = helper.cwd.join("injected-marker");
    let values = [
        "space argument",
        "Ω 空",
        "; touch injected-marker",
        "$(touch injected-marker)",
        "& echo injected>injected-marker",
        "C:\\native\\path with spaces",
        "a\"quoted\\value",
    ];
    let arguments = std::iter::once(OsString::from("snapshot"))
        .chain(values.iter().map(OsString::from))
        .collect();
    let spec = helper.spec(
        arguments,
        vec![("VFA_VISIBLE".into(), "selected-value".into())],
    );
    assert_eq!(spec.executable(), helper.executable.canonicalize().unwrap());
    assert_eq!(spec.working_directory(), helper.cwd.canonicalize().unwrap());
    assert!(
        std::env::var_os("PATH").is_some(),
        "fixture must have a genuine ambient PATH to reject"
    );
    let mut handle = SubprocessExecutor::spawn_spec(&spec, Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!(handle.wait().await.unwrap(), 0);
    let (stdout, stderr) = lines(&mut handle, 1, 0).await;
    assert!(stderr.is_empty());
    let actual: Value = serde_json::from_str(&stdout[0]).unwrap();
    assert_eq!(actual["arguments"], json!(values));
    let reported_cwd = PathBuf::from(actual["cwd"].as_str().unwrap());
    assert_eq!(
        reported_cwd.canonicalize().unwrap(),
        spec.working_directory()
    );
    assert_eq!(actual["environment"]["VFA_VISIBLE"], "selected-value");
    assert!(actual["environment"]["PATH"].is_null());
    assert!(actual["environment"]["AWS_SECRET_ACCESS_KEY"].is_null());
    assert!(!marker.exists(), "argv must not be evaluated as shell text");
    let json = spec.to_json().unwrap();
    assert_eq!(json["arguments"][1], "space argument");
    assert_eq!(json["environment"]["VFA_VISIBLE"], "selected-value");
}

#[tokio::test]
async fn typed_launch_returns_real_nonzero_exit_and_both_finite_streams() {
    let helper = Helper::compile();
    let spec = helper.spec(vec!["exit".into(), "37".into()], Vec::new());
    let mut handle = SubprocessExecutor::spawn_spec(&spec, Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!(handle.wait().await.unwrap(), 37);
    assert_eq!(handle.exit_code(), Some(37));
    let spec = helper.spec(vec!["streams".into()], Vec::new());
    let mut handle = SubprocessExecutor::spawn_spec(&spec, Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!(handle.wait().await.unwrap(), 0);
    let (stdout, stderr) = lines(&mut handle, 2, 2).await;
    assert_eq!(stdout, ["stdout-one", "stdout-two"]);
    assert_eq!(stderr, ["stderr-one", "stderr-two"]);
}

#[tokio::test]
async fn controlled_stdin_gate_proves_delay_and_child_is_reaped() {
    let helper = Helper::compile();
    let spec = helper.spec(vec!["gate-exit".into(), "23".into()], Vec::new());
    // Only this explicit fixture driver overrides typed stdin-null with a pipe.
    // No speculative stdin API is added to the old owned subprocess handle.
    let mut command = spec.command().unwrap();
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = tokio::process::Command::from(command).spawn().unwrap();
    let result: Result<(String, bool, String, i32), Box<dyn std::error::Error>> = async {
        let stdout = child.stdout.take().ok_or("missing stdout")?;
        let mut stdout = BufReader::new(stdout).lines();
        let ready = tokio::time::timeout(Duration::from_secs(10), stdout.next_line())
            .await??
            .ok_or("no READY")?;
        let pending = child.try_wait()?.is_none();
        let mut stdin = child.stdin.take().ok_or("missing stdin")?;
        stdin.write_all(b"G").await?;
        stdin.shutdown().await?;
        drop(stdin);
        let released = tokio::time::timeout(Duration::from_secs(10), stdout.next_line())
            .await??
            .ok_or("no RELEASED")?;
        let status = tokio::time::timeout(Duration::from_secs(10), child.wait()).await??;
        Ok((
            ready,
            pending,
            released,
            status.code().ok_or("no exit code")?,
        ))
    }
    .await;
    // On a failed observation, kill AND wait before asserting. A dropped future
    // or kill request alone would not prove even this direct child was reaped.
    if result.is_err() {
        let _ = child.kill().await;
        child
            .wait()
            .await
            .expect("failed helper observation must still reap its child");
    }
    let (ready, pending, released, code) = result.unwrap();
    assert_eq!(ready, "READY");
    assert!(
        pending,
        "child cannot exit before the explicit stdin release"
    );
    assert_eq!(released, "RELEASED");
    assert_eq!(code, 23);
    assert!(child.try_wait().unwrap().is_some());
}

#[test]
fn invalid_paths_arguments_and_environment_fail_without_execution() {
    let fixture = tempfile::tempdir().unwrap();
    let cwd = fixture.path().canonicalize().unwrap();
    let executable = std::env::current_exe().unwrap().canonicalize().unwrap();
    let resolve = |exe: &Path, args, dir: &Path, environment| {
        ProgramSpec::resolve(exe, args, dir, environment)
    };
    assert!(matches!(
        resolve(Path::new("relative"), Vec::new(), &cwd, Vec::new()),
        Err(ProgramError::RelativePath { .. })
    ));
    assert!(resolve(
        &cwd.join("missing-executable"),
        Vec::new(),
        &cwd,
        Vec::new()
    )
    .is_err());
    assert!(matches!(
        resolve(&cwd, Vec::new(), &cwd, Vec::new()),
        Err(ProgramError::NotExecutable)
    ));
    assert!(matches!(
        resolve(&executable, Vec::new(), &executable, Vec::new()),
        Err(ProgramError::NotDirectory)
    ));
    assert!(resolve(
        &executable,
        Vec::new(),
        &cwd.join("missing-directory"),
        Vec::new()
    )
    .is_err());
    assert!(matches!(
        resolve(
            &executable,
            vec![OsString::from("bad\0argument")],
            &cwd,
            Vec::new()
        ),
        Err(ProgramError::Nul { field: "argument" })
    ));
    let nul_path = cwd.join("bad\0path");
    assert!(matches!(
        resolve(&nul_path, Vec::new(), &cwd, Vec::new()),
        Err(ProgramError::Nul {
            field: "executable"
        })
    ));
    assert!(matches!(
        resolve(&executable, Vec::new(), &nul_path, Vec::new()),
        Err(ProgramError::Nul {
            field: "working directory"
        })
    ));
    for name in ["", "bad=name", "bad name", "1START", "Ω"] {
        assert!(matches!(
            resolve(
                &executable,
                Vec::new(),
                &cwd,
                vec![(name.into(), "value".into())]
            ),
            Err(ProgramError::InvalidEnvironmentName)
        ));
    }
    for name in [
        "AWS_SECRET_ACCESS_KEY",
        "github_token",
        "SSH_AUTH_SOCK",
        "NODE_OPTIONS",
        "NPM_CONFIG_USERCONFIG",
    ] {
        assert!(matches!(
            resolve(
                &executable,
                Vec::new(),
                &cwd,
                vec![(name.into(), "must-not-be-inherited".into())]
            ),
            Err(ProgramError::DeniedEnvironmentKey(_))
        ));
    }
    assert!(matches!(
        resolve(
            &executable,
            Vec::new(),
            &cwd,
            vec![("VISIBLE".into(), "bad\0value".into())]
        ),
        Err(ProgramError::Nul {
            field: "environment value"
        })
    ));
    assert!(matches!(
        resolve(
            &executable,
            Vec::new(),
            &cwd,
            vec![
                ("VISIBLE".into(), "one".into()),
                ("VISIBLE".into(), "two".into())
            ]
        ),
        Err(ProgramError::DuplicateEnvironmentKey(_))
    ));
    assert_eq!(
        std::fs::read_dir(cwd).unwrap().count(),
        0,
        "validation must not execute or create artifacts"
    );
}

#[test]
fn debug_hides_selected_environment_values_and_arguments() {
    let fixture = tempfile::tempdir().unwrap();
    let executable = std::env::current_exe().unwrap();
    let spec = ProgramSpec::resolve(
        &executable,
        vec!["argument-sentinel-never-log".into()],
        fixture.path(),
        vec![("VISIBLE".into(), "environment-sentinel-never-log".into())],
    )
    .unwrap();
    let diagnostic = format!("{spec:?}");
    assert!(diagnostic.contains("VISIBLE"));
    assert!(!diagnostic.contains("environment-sentinel-never-log"));
    assert!(!diagnostic.contains("argument-sentinel-never-log"));
}

#[cfg(unix)]
#[test]
fn native_non_unicode_paths_and_arguments_are_preserved_but_json_fails() {
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::PermissionsExt;
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().canonicalize().unwrap();
    let executable = root.join(OsString::from_vec(b"native-executable-\xff".to_vec()));
    std::fs::write(&executable, b"native identity fixture; never executed\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let spec = ProgramSpec::resolve(&executable, Vec::new(), &root, Vec::new()).unwrap();
    assert_eq!(spec.executable().as_os_str(), executable.as_os_str());
    assert!(matches!(
        spec.to_json(),
        Err(ProgramError::Unrepresentable {
            field: "executable"
        })
    ));
    let invalid = OsString::from_vec(b"native-argument-\xff".to_vec());
    let spec = ProgramSpec::resolve(
        &std::env::current_exe().unwrap(),
        vec![invalid.clone()],
        &root,
        Vec::new(),
    )
    .unwrap();
    assert_eq!(spec.arguments(), [invalid]);
    assert!(matches!(
        spec.to_json(),
        Err(ProgramError::Unrepresentable { field: "argument" })
    ));
    let directory = root.join(OsString::from_vec(b"native-directory-\xfe".to_vec()));
    std::fs::create_dir(&directory).unwrap();
    let spec = ProgramSpec::resolve(
        &std::env::current_exe().unwrap(),
        Vec::new(),
        &directory,
        Vec::new(),
    )
    .unwrap();
    assert_eq!(spec.working_directory(), directory);
    assert!(matches!(
        spec.to_json(),
        Err(ProgramError::Unrepresentable {
            field: "working directory"
        })
    ));
    let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(&executable, permissions).unwrap();
    assert!(matches!(
        ProgramSpec::resolve(&executable, Vec::new(), &root, Vec::new()),
        Err(ProgramError::NotExecutable)
    ));
}

#[cfg(windows)]
#[test]
fn windows_batch_files_are_not_admitted_as_shell_free_executables() {
    let fixture = tempfile::tempdir().unwrap();
    let executable = fixture.path().join("never-invoke.cmd");
    std::fs::write(&executable, "@echo must-not-run\r\n").unwrap();
    assert!(matches!(
        ProgramSpec::resolve(&executable, Vec::new(), fixture.path(), Vec::new()),
        Err(ProgramError::NotExecutable)
    ));
}
