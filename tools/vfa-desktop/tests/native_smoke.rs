//! Explicit real-window bootstrap. Not the later runtime/process admission gate.

use gpui_kit::{AppContext as _, QuitMode, WindowOptions};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn file_digest(path: &Path) -> Result<String, String> {
    let mut source = File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = source
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            return Ok(format!("{:x}", hash.finalize()));
        }
        hash.update(&buffer[..count]);
    }
}

fn write_new(path: &Path, value: &Value) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("Cannot create evidence: {error}"))?;
    serde_json::to_writer_pretty(&mut file, value).map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

fn compatible_host(backend: &str) -> bool {
    match backend {
        "x11" | "wayland" => cfg!(all(target_os = "linux", target_arch = "x86_64")),
        "macos" => cfg!(all(target_os = "macos", target_arch = "aarch64")),
        "windows" => cfg!(all(
            target_os = "windows",
            target_arch = "x86_64",
            target_env = "msvc"
        )),
        _ => false,
    }
}

fn native_child(backend: String, output: PathBuf) -> Result<(), String> {
    if !compatible_host(&backend) {
        return Err("Requested backend does not match this native host".into());
    }
    let observation = Rc::new(RefCell::new(None));
    let captured = observation.clone();
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .with_quit_mode(QuitMode::Explicit)
        .run(move |cx| {
            gpui_kit::init(cx);
            let opened = vfa_desktop::open_bootstrap(cx, WindowOptions::default());
            let (handle, _) = match opened {
                Ok(opened) => opened,
                Err(error) => {
                    *captured.borrow_mut() = Some(Err(error));
                    cx.quit();
                    return;
                }
            };
            let failed = captured.clone();
            if let Err(error) = handle.update(cx, move |_, window, cx| {
                let observed = match HasWindowHandle::window_handle(window).map(|h| h.as_raw()) {
                    Ok(RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_)) => "x11",
                    Ok(RawWindowHandle::Wayland(_)) => "wayland",
                    Ok(RawWindowHandle::AppKit(_)) => "macos",
                    Ok(RawWindowHandle::Win32(_)) => "windows",
                    _ => "unsupported",
                };
                if observed != backend {
                    *captured.borrow_mut() = Some(Err(format!(
                        "Requested {backend}, actual native handle is {observed}"
                    )));
                    cx.quit();
                    return;
                }
                // In this snapshot callbacks run BEFORE draw/present, despite
                // the API comment. Force a draw turn, then observe the next turn.
                window.on_next_frame(move |window, _| {
                    window.refresh();
                    window.on_next_frame(move |window, cx| {
                        let scene_primitives = window.rendered_scene().len();
                        if scene_primitives == 0
                            || (cfg!(target_os = "linux") && window.gpu_specs().is_none())
                        {
                            *captured.borrow_mut() =
                                Some(Err("No native scene or Linux renderer was observed".into()));
                            cx.quit();
                            return;
                        }
                        let gpu = window.gpu_specs().map(|gpu| {
                            json!({
                                "device": gpu.device_name, "driver": gpu.driver_name,
                                "driver_info": gpu.driver_info,
                                "software_emulated": gpu.is_software_emulated,
                            })
                        });
                        *captured.borrow_mut() = Some(Ok(json!({
                            "observed_backend": observed,
                            "platform": format!("{:?}", cx.platform_info()),
                            "compositor": cx.compositor_name(),
                            "native_frame_progress_observed": true,
                            "scene_primitives": scene_primitives,
                            "gpu": gpu,
                        })));
                        window.remove_window();
                        cx.quit();
                    });
                });
            }) {
                *failed.borrow_mut() = Some(Err(format!("Native window update failed: {error}")));
                cx.quit();
            }
        });
    let mut value = observation
        .take()
        .ok_or("Event loop returned without observing a native frame")??;
    value["event_loop_returned"] = json!(true);
    write_new(&output, &value)
}

fn text(value: OsString) -> Result<String, String> {
    value
        .into_string()
        .map_err(|_| "Non-UTF8 option value".into())
}

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() == Some(std::ffi::OsStr::new("--native-child")) {
        let backend = text(args.next().ok_or("Missing child backend")?)?;
        let output = PathBuf::from(args.next().ok_or("Missing child evidence destination")?);
        if args.next().is_some() {
            return Err("Unexpected native child argument".into());
        }
        return native_child(backend, output);
    }
    let mut args = std::env::args_os().skip(1);
    let (mut scenario, mut backend, mut evidence) = (None, None, None);
    while let Some(arg) = args.next() {
        let destination = match arg.to_str() {
            Some("--scenario") if scenario.is_none() => &mut scenario,
            Some("--backend") if backend.is_none() => &mut backend,
            Some("--evidence-dir") if evidence.is_none() => &mut evidence,
            _ => {
                return Err(
                    "Expected unique --scenario, --backend and --evidence-dir options".into(),
                );
            }
        };
        *destination = Some(args.next().ok_or("Missing option value")?);
    }
    if scenario.as_deref() != Some(std::ffi::OsStr::new("bootstrap")) {
        return Err("Only --scenario bootstrap is implemented; admission is a later task".into());
    }
    let backend = text(backend.ok_or("Missing --backend")?)?;
    if !compatible_host(&backend) {
        return Err(
            "Choose x11/wayland, macos ARM64 or windows x64 MSVC on that native host".into(),
        );
    }
    let directory = PathBuf::from(evidence.ok_or("Missing --evidence-dir")?);
    fs::create_dir_all(&directory).map_err(|error| format!("Evidence directory: {error}"))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let prefix = format!("bootstrap-{backend}-{}-{stamp}", std::process::id());
    let record = directory.join(format!("{prefix}-observation.json"));
    let log = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(format!("{prefix}.log")))
        .map_err(|e| e.to_string())?;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut command = Command::new(&executable);
    command
        .args([
            OsString::from("--native-child"),
            OsString::from(&backend),
            record.as_os_str().to_owned(),
        ])
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log);
    // Disable the other Linux selector rather than silently accepting XWayland.
    if backend == "x11" {
        command.env_remove("WAYLAND_DISPLAY");
    }
    if backend == "wayland" {
        command.env_remove("DISPLAY");
    }
    let started = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|e| format!("Cannot launch native probe: {e}"))?;
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        if started.elapsed() >= Duration::from_secs(10) {
            timed_out = true;
            child.kill().map_err(|e| e.to_string())?;
            break child.wait().map_err(|e| e.to_string())?;
        }
        thread::sleep(Duration::from_millis(20));
    };
    let observation = fs::read(&record)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    let passed = !timed_out
        && status.success()
        && observation.as_ref().is_some_and(|value| {
            value["observed_backend"] == backend
                && value["native_frame_progress_observed"] == true
                && value["scene_primitives"]
                    .as_u64()
                    .is_some_and(|count| count > 0)
                && value["event_loop_returned"] == true
        });
    let result = json!({
        "scenario": "bootstrap", "outcome": if passed { "PASS" } else { "FAIL" },
        "requested_backend": backend, "observation": observation,
        "timed_out": timed_out, "child_exit_code": status.code(),
        "elapsed_ms": started.elapsed().as_millis(),
        "binary_sha256": file_digest(&executable)?,
        "lockfile_sha256": digest(include_bytes!("../Cargo.lock")),
        "bootstrap_source_sha256": digest(include_bytes!("../src/lib.rs")),
        "toolkit_provenance_sha256": digest(include_bytes!("../provenance/toolkit.json")),
        "platform_os": std::env::consts::OS, "platform_arch": std::env::consts::ARCH,
        "observed_at_unix_ns": stamp,
        "limits": "Native handle, nonempty scene and intervening draw/present turn only; not pixels/compositor display, AT, IME, clipboard or runtime/process admission",
    });
    write_new(&directory.join(format!("{prefix}.json")), &result)?;
    if passed {
        Ok(())
    } else {
        Err("Native bootstrap failed; inspect selected evidence directory".into())
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("native_smoke: {error}");
        std::process::exit(1);
    }
}
