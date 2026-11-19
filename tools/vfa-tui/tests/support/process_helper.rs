//! Standalone std-only controlled helper, compiled into a test-owned directory.
//! Not a Cargo target. Finite joined descendants only; no OS cleanup acceptance.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::{Command, ExitCode, Stdio};

fn tree_record(record: String) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(format!("{record}\n").as_bytes())
        .and_then(|_| stdout.flush())
        .map_err(|error| error.to_string())
}

fn finite_tree(depth: u8, code: u8) -> Result<u8, String> {
    let pid = std::process::id();
    tree_record(format!(
        "{{\"event\":\"started\",\"pid\":{pid},\"depth\":{depth}}}"
    ))?;
    if depth > 0 {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut child = Command::new(executable)
            .args(["tree".to_owned(), (depth - 1).to_string(), code.to_string()])
            .stdin(Stdio::null())
            .spawn()
            .map_err(|error| error.to_string())?;
        let child_pid = child.id();
        // A failed observer must not drop an un-waited Child. These children
        // are finite and cannot wait for input; join even if stdout is closed.
        let spawned = tree_record(format!(
            "{{\"event\":\"spawned\",\"pid\":{pid},\"depth\":{depth},\"child\":{child_pid}}}"
        ));
        let status = child.wait().map_err(|error| error.to_string())?;
        spawned?;
        let observed = status.code().ok_or("child did not exit normally")?;
        tree_record(format!(
            "{{\"event\":\"joined\",\"pid\":{pid},\"depth\":{depth},\"child\":{child_pid},\"code\":{observed}}}"
        ))?;
        if observed != i32::from(code) {
            return Err("unexpected child exit".into());
        }
    }
    tree_record(format!(
        "{{\"event\":\"exiting\",\"pid\":{pid},\"depth\":{depth},\"code\":{code}}}"
    ))?;
    Ok(code)
}

fn quote(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                use std::fmt::Write;
                write!(output, "\\u{:04x}", character as u32).unwrap();
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

fn text(value: &OsString) -> Result<&str, String> {
    value
        .to_str()
        .ok_or_else(|| "unrepresentable helper input".into())
}

// Bounded, finite output pressure fixtures. Existing process/tree modes below
// remain unchanged; these modes never spawn descendant processes.
fn output_size(value: &OsString, max: usize) -> Result<usize, String> {
    let count = text(value)?
        .parse::<usize>()
        .map_err(|_| "invalid output size")?;
    if count == 0 || count > max {
        return Err("output size outside finite limit".into());
    }
    Ok(count)
}

fn output_blocks(mut stream: impl Write, count: usize, lines: bool) -> io::Result<()> {
    let mut block = [b'x'; 4096];
    if lines {
        block[4095] = b'\n';
    }
    for _ in 0..count / block.len() {
        stream.write_all(&block)?;
    }
    stream.write_all(&block[..count % block.len()])?;
    stream.flush()
}

fn output_export(mode: &str) -> Result<u8, String> {
    if !["valid", "overflow", "invalid", "hostile", "wire-overflow"].contains(&mode) {
        return Err("invalid export fixture mode".into());
    }
    io::stderr()
        .write_all(b"export fixture\n")
        .map_err(|e| e.to_string())?;
    let pressure = std::thread::spawn(|| output_blocks(io::stderr().lock(), 1024 * 1024, true));
    let result = (|| -> io::Result<()> {
        let mut out = io::stdout().lock();
        let digest = "a".repeat(64);
        let hash = "b".repeat(64);
        let count = if mode == "wire-overflow" { 200 } else { 1 };
        writeln!(out, "{{\"type\":\"apply-start\",\"protocol\":\"vfa-export/1\",\"plan_digest\":\"{digest}\",\"seq\":1,\"file_count\":{count}}}")?;
        for index in 0..count {
            let path = if index == 0 {
                "result.txt".into()
            } else {
                format!("f{index}")
            };
            let record = format!("{{\"type\":\"file-outcome\",\"protocol\":\"vfa-export/1\",\"plan_digest\":\"{digest}\",\"seq\":{},\"file_index\":{index},\"path\":\"{path}\",\"status\":\"WRITTEN\",\"final\":{{\"sha256\":\"{hash}\",\"size\":7,\"mode\":420}}}}", index + 2);
            out.write_all(record.as_bytes())?;
            if mode == "wire-overflow" {
                let spaces = [b' '; 4096];
                let padding = 65535 - record.len();
                for _ in 0..padding / spaces.len() {
                    out.write_all(&spaces)?;
                }
                out.write_all(&spaces[..padding % spaces.len()])?;
            }
            out.write_all(b"\n")?;
        }
        match mode {
            "overflow" => output_blocks(&mut out, 131072, false)?,
            "invalid" => out.write_all(b"not-json\n")?,
            "hostile" => {
                out.write_all(b"{\"type\":\"apply-summary\",\"")?;
                for _ in 0..10000 { out.write_all(b"\\u007f")?; }
                out.write_all(b"\":0}\n")?;
            }
            "valid" => writeln!(out, "{{\"type\":\"apply-summary\",\"protocol\":\"vfa-export/1\",\"plan_digest\":\"{digest}\",\"seq\":3,\"status\":\"SUCCEEDED\",\"planned\":1,\"written\":1,\"failed_index\":null,\"unattempted\":0,\"uncertain\":false,\"code\":null}}")?,
            _ => {}
        }
        out.flush()
    })();
    let pressure = pressure.join().map_err(|_| "output writer panicked")?;
    result.and(pressure).map_err(|e| e.to_string())?;
    Ok(0)
}

fn run() -> Result<u8, String> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let mode = arguments.first().ok_or("missing helper mode")?;
    match text(mode)? {
        "output-dual" if arguments.len() == 3 => {
            let count = output_size(&arguments[1], 64 * 1024 * 1024)?;
            let lines = match text(&arguments[2])? {
                "lines" => true,
                "plain" => false,
                _ => return Err("invalid output shape".into()),
            };
            let out = std::thread::spawn(move || output_blocks(io::stdout().lock(), count, lines));
            let err = std::thread::spawn(move || output_blocks(io::stderr().lock(), count, lines));
            let out = out.join().map_err(|_| "stdout writer panicked")?;
            let err = err.join().map_err(|_| "stderr writer panicked")?;
            out.and(err).map_err(|e| e.to_string())?;
            Ok(0)
        }
        "output-empty" if arguments.len() == 2 => {
            let count = output_size(&arguments[1], 100000)?;
            let mut out = io::stdout().lock();
            for _ in 0..count {
                out.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            Ok(0)
        }
        "output-unicode" if arguments.len() == 1 => {
            let mut out = io::stdout().lock();
            out.write_all("hello Ω ".as_bytes())
                .map_err(|e| e.to_string())?;
            for byte in "🦀\n".as_bytes() {
                out.write_all(&[*byte])
                    .and_then(|_| out.flush())
                    .map_err(|e| e.to_string())?;
            }
            out.write_all(b"x\xffy\n").map_err(|e| e.to_string())?;
            out.write_all(&[b'a'; 4094]).map_err(|e| e.to_string())?;
            out.write_all("🦀\n".as_bytes())
                .map_err(|e| e.to_string())?;
            Ok(0)
        }
        "output-pause" if arguments.len() == 2 => {
            let millis = output_size(&arguments[1], 2000)?;
            std::thread::sleep(std::time::Duration::from_millis(millis as u64));
            Ok(0)
        }
        "output-export" if arguments.len() == 2 => output_export(text(&arguments[1])?),
        "output-marker" if arguments.len() == 1 => {
            std::fs::write("must-not-launch", b"launched").map_err(|e| e.to_string())?;
            Ok(0)
        }
        "tree" if arguments.len() == 3 => {
            let depth: u8 = text(&arguments[1])?
                .parse()
                .map_err(|_| "invalid tree depth")?;
            if depth > 3 {
                return Err("tree depth exceeds three descendant edges".into());
            }
            let code: u8 = text(&arguments[2])?
                .parse()
                .map_err(|_| "invalid exit code")?;
            finite_tree(depth, code)
        }
        "snapshot" => {
            let argv = arguments
                .iter()
                .skip(1)
                .map(|value| text(value).map(quote))
                .collect::<Result<Vec<_>, _>>()?
                .join(",");
            let directory = std::env::current_dir().map_err(|error| error.to_string())?;
            let cwd = directory.to_str().ok_or("unrepresentable helper cwd")?;
            let mut environment = Vec::new();
            for key in ["VFA_VISIBLE", "PATH", "AWS_SECRET_ACCESS_KEY"] {
                let value = match std::env::var_os(key) {
                    Some(value) => quote(text(&value)?),
                    None => "null".into(),
                };
                environment.push(format!("{}:{value}", quote(key)));
            }
            println!(
                "{{\"arguments\":[{argv}],\"cwd\":{},\"environment\":{{{}}}}}",
                quote(cwd),
                environment.join(",")
            );
            Ok(0)
        }
        "streams" if arguments.len() == 1 => {
            println!("stdout-one");
            eprintln!("stderr-one");
            println!("stdout-two");
            eprintln!("stderr-two");
            Ok(0)
        }
        "exit" | "gate-exit" if arguments.len() == 2 => {
            let code: u8 = text(&arguments[1])?
                .parse()
                .map_err(|_| "invalid exit code")?;
            if text(mode)? == "gate-exit" {
                println!("READY");
                io::stdout().flush().map_err(|error| error.to_string())?;
                let mut release = [0u8; 1];
                io::stdin()
                    .read_exact(&mut release)
                    .map_err(|error| error.to_string())?;
                if release != [b'G'] {
                    return Err("invalid release byte".into());
                }
                println!("RELEASED");
                io::stdout().flush().map_err(|error| error.to_string())?;
            }
            Ok(code)
        }
        _ => Err("unknown helper mode/arguments".into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("controlled process helper: {error}");
            ExitCode::from(64)
        }
    }
}
