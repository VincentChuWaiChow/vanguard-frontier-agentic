//! Standalone std-only controlled helper, compiled into a test-owned directory.
//! Not a Cargo target. No orphan/descendant or OS cleanup acceptance mode.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::process::ExitCode;

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

fn run() -> Result<u8, String> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let mode = arguments.first().ok_or("missing helper mode")?;
    match text(mode)? {
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
