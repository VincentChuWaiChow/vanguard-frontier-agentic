//! Native process tests, using only explicitly selected temporary fixture roots.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use vfa_tui::engine::profile::{ProfileError, ProfileOwner, ProfilePaths};

#[cfg(unix)]
use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};

const JOURNAL: &[u8] = b"nonterminal job belongs to live first session; do not recover";
const PREFERENCES: &[u8] = b"private preferences: unchanged";
const LOCK_CONTENT: &[u8] = b"stale PID is not ownership proof; never truncate this file";
const CHILD_MODE: &str = "VFA_PROFILE_OWNERSHIP_TEST_MODE";
const CHILD_ROOT: &str = "VFA_PROFILE_OWNERSHIP_TEST_ROOT";
const CHILD_CONTROL: &str = "VFA_PROFILE_OWNERSHIP_TEST_CONTROL";

fn fixture() -> (TempDir, PathBuf) {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture
        .path()
        .canonicalize()
        .unwrap()
        .join("profile Ω 空 spaces");
    (fixture, root)
}

fn prepared(root: &Path) -> ProfilePaths {
    let paths = ProfilePaths::resolve(Some(root)).unwrap();
    let owner = ProfileOwner::acquire(paths.clone()).unwrap();
    fs::write(owner.paths().journal(), JOURNAL).unwrap();
    fs::write(owner.paths().preferences(), PREFERENCES).unwrap();
    drop(owner);
    fs::write(paths.lock_file(), LOCK_CONTENT).unwrap();
    paths
}

fn child_command(root: &Path, mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "ownership_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_MODE, mode)
        .env(CHILD_ROOT, root);
    command
}

fn child_succeeds(root: &Path, mode: &str) {
    let output = bounded_output(child_command(root, mode));
    assert!(
        output.status.success(),
        "child {mode} failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn bounded_output(mut command: Command) -> Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!(
                "profile child timed out: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

struct OwnerProcess {
    child: Child,
    control: PathBuf,
}

impl OwnerProcess {
    fn start(root: &Path, control: PathBuf) -> Self {
        let child = child_command(root, "hold")
            .env(CHILD_CONTROL, &control)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let mut process = Self { child, control };
        process.wait_for("owned");
        process
    }

    fn wait_for(&mut self, expected: &str) {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if fs::read_to_string(&self.control).ok().as_deref() == Some(expected) {
                return;
            }
            assert!(
                self.child.try_wait().unwrap().is_none(),
                "owner exited early"
            );
            assert!(Instant::now() < deadline, "owner handshake timed out");
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn release(&mut self) {
        let input = self.child.stdin.as_mut().unwrap();
        writeln!(input, "release").unwrap();
        input.flush().unwrap();
        self.wait_for("released");
        assert!(self.child.try_wait().unwrap().is_none());
    }

    fn finish(&mut self) {
        self.child.stdin.take();
        assert!(self.child.wait().unwrap().success());
    }
}

impl Drop for OwnerProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn assert_data_unchanged(paths: &ProfilePaths) {
    assert_eq!(fs::read(paths.journal()).unwrap(), JOURNAL);
    assert_eq!(fs::read(paths.preferences()).unwrap(), PREFERENCES);
    assert!(!paths.root().join("recovered-by-second-instance").exists());
}

fn assert_same_lock(paths: &ProfilePaths, before: &fs::Metadata) {
    let after = fs::symlink_metadata(paths.lock_file()).unwrap();
    assert_eq!(before.len(), after.len());
    assert_eq!(before.modified().unwrap(), after.modified().unwrap());
    #[cfg(unix)]
    {
        assert_eq!(before.dev(), after.dev());
        assert_eq!(before.ino(), after.ino());
    }
}

#[test]
fn canonical_private_paths_and_owner_are_native_and_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ProfileOwner>();
    let (_fixture, root) = fixture();
    let paths = ProfilePaths::resolve(Some(&root)).unwrap();
    assert_eq!(paths.root(), root.canonicalize().unwrap());
    assert_eq!(paths.journal(), paths.root().join("operations.sqlite"));
    assert_eq!(paths.preferences(), paths.root().join("preferences.json"));
    assert!(!paths.journal().exists());
    assert!(!paths.preferences().exists());
    assert!(
        !paths.lock_file().exists(),
        "directory creation is not ownership"
    );
    let alias = root.join("..").join(root.file_name().unwrap());
    let alias_paths = ProfilePaths::resolve(Some(&alias)).unwrap();
    assert_eq!(alias_paths, paths);
    let owner = ProfileOwner::acquire(paths.clone()).unwrap();
    assert_eq!(owner.paths(), &paths);
    assert!(matches!(
        ProfileOwner::acquire(alias_paths),
        Err(ProfileError::Busy { .. })
    ));
    #[cfg(unix)]
    {
        assert_eq!(fs::metadata(paths.root()).unwrap().mode() & 0o777, 0o700);
        assert_eq!(
            fs::metadata(paths.lock_file()).unwrap().mode() & 0o777,
            0o600
        );
        // SAFETY: geteuid has no preconditions or side effects.
        assert_eq!(fs::metadata(paths.root()).unwrap().uid(), unsafe {
            libc::geteuid()
        });
    }
}

#[test]
fn two_process_exclusion_and_normal_drop_preserve_owned_data() {
    let (fixture, root) = fixture();
    let paths = prepared(&root);
    let before = fs::metadata(paths.lock_file()).unwrap();
    let mut first = OwnerProcess::start(&root, fixture.path().join("owner state"));
    let alias = root.join("..").join(root.file_name().unwrap());
    child_succeeds(&alias, "busy");
    assert_data_unchanged(&paths);
    assert_same_lock(&paths, &before);
    first.release();
    child_succeeds(&root, "available");
    assert_data_unchanged(&paths);
    assert_eq!(fs::read(paths.lock_file()).unwrap(), LOCK_CONTENT);
    assert_same_lock(&paths, &before);
    first.finish();
}

#[test]
fn forced_owner_death_releases_the_same_os_lock() {
    let (fixture, root) = fixture();
    let paths = prepared(&root);
    let before = fs::metadata(paths.lock_file()).unwrap();
    let mut first = OwnerProcess::start(&root, fixture.path().join("owner state"));
    child_succeeds(&root, "busy");
    first.child.kill().unwrap();
    assert!(!first.child.wait().unwrap().success());
    child_succeeds(&root, "available");
    assert_data_unchanged(&paths);
    assert_eq!(fs::read(paths.lock_file()).unwrap(), LOCK_CONTENT);
    assert_same_lock(&paths, &before);
}

#[test]
fn invalid_roots_never_fallback_or_modify_existing_content() {
    let (fixture, root) = fixture();
    assert!(matches!(
        ProfilePaths::resolve(Some(Path::new("relative profile"))),
        Err(ProfileError::RelativeRoot(_))
    ));
    assert!(matches!(
        ProfilePaths::resolve(Some(Path::new(""))),
        Err(ProfileError::RelativeRoot(_))
    ));
    fs::write(&root, b"not a directory").unwrap();
    assert!(ProfilePaths::resolve(Some(&root)).is_err());
    assert!(ProfilePaths::resolve(Some(&root.join("child"))).is_err());
    assert_eq!(fs::read(&root).unwrap(), b"not a directory");
    assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 1);
}

#[test]
fn nonregular_lock_is_rejected_without_deleting_it() {
    let (_fixture, root) = fixture();
    let paths = ProfilePaths::resolve(Some(&root)).unwrap();
    fs::create_dir(paths.lock_file()).unwrap();
    let sentinel = paths.lock_file().join("keep");
    fs::write(&sentinel, b"unchanged").unwrap();
    assert!(matches!(
        ProfileOwner::acquire(paths.clone()),
        Err(ProfileError::UnsafeLock { .. })
    ));
    assert_eq!(fs::read(sentinel).unwrap(), b"unchanged");
    assert!(paths.lock_file().is_dir());
}

#[cfg(unix)]
#[test]
fn unsafe_unix_modes_symlinks_and_hardlinks_are_rejected_without_repairs() {
    let (fixture, root) = fixture();
    let parent_mode = fs::metadata(fixture.path()).unwrap().mode();
    let paths = ProfilePaths::resolve(Some(&root)).unwrap();
    for mode in [0o755, 0o500] {
        fs::set_permissions(&root, fs::Permissions::from_mode(mode)).unwrap();
        assert!(ProfilePaths::resolve(Some(&root)).is_err());
        assert_eq!(fs::metadata(&root).unwrap().mode() & 0o777, mode);
        assert!(!paths.lock_file().exists());
    }
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(fs::metadata(fixture.path()).unwrap().mode(), parent_mode);
    let root_alias = fixture.path().join("symlink profile");
    symlink(&root, &root_alias).unwrap();
    assert!(ProfilePaths::resolve(Some(&root_alias)).is_err());
    let target = fixture.path().join("foreign lock target");
    fs::write(&target, b"do not modify target").unwrap();
    symlink(&target, paths.lock_file()).unwrap();
    assert!(matches!(
        ProfileOwner::acquire(paths.clone()),
        Err(ProfileError::UnsafeLock { .. })
    ));
    assert_eq!(fs::read(&target).unwrap(), b"do not modify target");
    assert!(fs::symlink_metadata(paths.lock_file())
        .unwrap()
        .file_type()
        .is_symlink());
    fs::remove_file(paths.lock_file()).unwrap(); // Fixture teardown, never an owned lock.
    fs::hard_link(&target, paths.lock_file()).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(
        ProfileOwner::acquire(paths.clone()),
        Err(ProfileError::UnsafeLock { .. })
    ));
    assert_eq!(fs::metadata(paths.lock_file()).unwrap().nlink(), 2);
    fs::remove_file(paths.lock_file()).unwrap();
    fs::write(paths.lock_file(), LOCK_CONTENT).unwrap();
    fs::set_permissions(paths.lock_file(), fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(
        ProfileOwner::acquire(paths.clone()),
        Err(ProfileError::UnsafeLock { .. })
    ));
    assert_eq!(
        fs::metadata(paths.lock_file()).unwrap().mode() & 0o777,
        0o644
    );
    assert_eq!(fs::read(paths.lock_file()).unwrap(), LOCK_CONTENT);
}

#[cfg(target_os = "linux")]
#[test]
fn linux_discovery_requires_explicit_absolute_xdg_without_home_or_tmp_fallback() {
    let (fixture, root) = fixture();
    for xdg in [None, Some(""), Some("relative data")] {
        let mut command = child_command(&root, "discovery-error");
        command
            .env_remove("XDG_DATA_HOME")
            .env("HOME", fixture.path().join("unused home"))
            .env("TMPDIR", fixture.path().join("unused tmp"));
        if let Some(value) = xdg {
            command.env("XDG_DATA_HOME", value);
        }
        let output = bounded_output(command);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read_dir(fixture.path()).unwrap().count(), 0);
    }
    let data = fixture
        .path()
        .canonicalize()
        .unwrap()
        .join("XDG private data Ω");
    let mut command = child_command(&root, "discovery-valid");
    command.env("XDG_DATA_HOME", &data);
    let output = bounded_output(command);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(data.join("vanguard-frontier-agentic/desktop").is_dir());
    let mut command = child_command(&root, "explicit-override");
    command.env("XDG_DATA_HOME", "invalid relative");
    let output = bounded_output(command);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.is_dir());
    assert!(!fixture.path().join("unused home").exists());
    assert!(!fixture.path().join("unused tmp").exists());
}

/// The same integration test executable is the only child helper; no helper
/// binary or automatic real-user profile access is installed or published.
#[test]
fn ownership_child() {
    let Some(mode) = std::env::var_os(CHILD_MODE) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os(CHILD_ROOT).unwrap());
    match mode.to_str().unwrap() {
        "discovery-error" => {
            assert!(ProfilePaths::resolve(None).is_err());
        }
        "discovery-valid" => {
            let expected = PathBuf::from(std::env::var_os("XDG_DATA_HOME").unwrap())
                .join("vanguard-frontier-agentic/desktop");
            let paths = ProfilePaths::resolve(None).unwrap();
            assert_eq!(paths.root(), expected.canonicalize().unwrap());
            assert!(!paths.lock_file().exists());
        }
        "explicit-override" => {
            ProfilePaths::resolve(Some(&root)).unwrap();
        }
        "hold" => {
            let paths = ProfilePaths::resolve(Some(&root)).unwrap();
            let owner = ProfileOwner::acquire(paths).unwrap();
            let control = PathBuf::from(std::env::var_os(CHILD_CONTROL).unwrap());
            fs::write(&control, "owned").unwrap();
            let mut input = io::stdin().lock();
            let mut line = String::new();
            input.read_line(&mut line).unwrap();
            assert_eq!(line.trim(), "release");
            drop(owner);
            fs::write(control, "released").unwrap();
            line.clear();
            input.read_line(&mut line).unwrap();
        }
        "busy" => {
            let paths = ProfilePaths::resolve(Some(&root)).unwrap();
            match ProfileOwner::acquire(paths) {
                Err(ProfileError::Busy { .. }) => {}
                Ok(owner) => {
                    // A broken lock mutates all three sentinels and fails. The
                    // parent independently checks no recovery/write occurred.
                    fs::write(owner.paths().journal(), b"incorrect recovered job").unwrap();
                    fs::write(owner.paths().preferences(), b"incorrect preferences").unwrap();
                    fs::write(
                        owner.paths().root().join("recovered-by-second-instance"),
                        b"incorrect recovery",
                    )
                    .unwrap();
                    panic!("second process acquired a live profile");
                }
                Err(error) => panic!("expected named Busy, got {error}"),
            }
        }
        "available" => {
            let paths = ProfilePaths::resolve(Some(&root)).unwrap();
            let owner = ProfileOwner::acquire(paths).unwrap();
            assert_data_unchanged(owner.paths());
            let paths = owner.paths().clone();
            drop(owner);
            // Windows byte-range locks intentionally reject reads through any
            // second handle; inspect lock contents only after ownership ends.
            assert_eq!(fs::read(paths.lock_file()).unwrap(), LOCK_CONTENT);
        }
        other => panic!("unknown child mode: {other}"),
    }
}
