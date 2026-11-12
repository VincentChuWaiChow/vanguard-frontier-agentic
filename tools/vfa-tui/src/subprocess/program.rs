//! Native, shell-free launch inputs. This is not a trust grant or a sandbox.
//!
//! Environment entries supplied by the caller are the complete allowlist; no
//! ambient environment is added. Known secret/credential keys are rejected, not
//! silently restored. This is not comprehensive environment or DLP protection.

use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::security::redact::is_secret_env_var;

#[derive(Debug, thiserror::Error)]
pub enum ProgramError {
    #[error("{field} must be an absolute native path")]
    RelativePath { field: &'static str },
    #[error("{field} contains an embedded NUL")]
    Nul { field: &'static str },
    #[error("cannot resolve or inspect {field}: {source}")]
    Path {
        field: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("resolved executable must be a regular executable file")]
    NotExecutable,
    #[error("resolved working directory must be a directory")]
    NotDirectory,
    #[error("environment name must be a portable ASCII identifier")]
    InvalidEnvironmentName,
    #[error("known secret/credential environment key is denied: {0}")]
    DeniedEnvironmentKey(String),
    #[error("duplicate selected environment key: {0}")]
    DuplicateEnvironmentKey(String),
    #[error("{field} cannot be represented as a JSON string without loss")]
    Unrepresentable { field: &'static str },
}

#[derive(Clone)]
pub struct ProgramSpec {
    executable: PathBuf,
    arguments: Vec<OsString>,
    working_directory: PathBuf,
    environment: Vec<(OsString, OsString)>,
}

impl fmt::Debug for ProgramSpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Diagnostics never include environment values or argument contents.
        formatter
            .debug_struct("ProgramSpec")
            .field("executable", &self.executable)
            .field("working_directory", &self.working_directory)
            .field("argument_count", &self.arguments.len())
            .field(
                "environment_keys",
                &self
                    .environment
                    .iter()
                    .map(|(key, _)| key)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl ProgramSpec {
    /// Resolve explicit executable/cwd paths without spawning or probing versions.
    /// Native paths/argv are not subjected to identifier/shell-metacharacter rules.
    pub fn resolve(
        executable: &Path,
        arguments: Vec<OsString>,
        working_directory: &Path,
        environment: Vec<(OsString, OsString)>,
    ) -> Result<Self, ProgramError> {
        let executable = resolve_path(executable, "executable")?;
        let working_directory = resolve_path(working_directory, "working directory")?;
        validate_paths(&executable, &working_directory)?;
        for argument in &arguments {
            reject_nul(argument, "argument")?;
        }
        let mut names = BTreeSet::new();
        for (name, value) in &environment {
            reject_nul(name, "environment name")?;
            reject_nul(value, "environment value")?;
            let name = name.to_str().ok_or(ProgramError::InvalidEnvironmentName)?;
            if name.is_empty()
                || !(name.as_bytes()[0].is_ascii_alphabetic() || name.as_bytes()[0] == b'_')
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return Err(ProgramError::InvalidEnvironmentName);
            }
            if is_secret_env_var(name) {
                return Err(ProgramError::DeniedEnvironmentKey(name.to_owned()));
            }
            #[cfg(windows)]
            let identity = name.to_ascii_uppercase();
            #[cfg(not(windows))]
            let identity = name.to_owned();
            if !names.insert(identity) {
                return Err(ProgramError::DuplicateEnvironmentKey(name.to_owned()));
            }
        }
        Ok(Self {
            executable,
            arguments,
            working_directory,
            environment,
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }
    pub fn arguments(&self) -> &[OsString] {
        &self.arguments
    }
    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }
    pub fn environment(&self) -> &[(OsString, OsString)] {
        &self.environment
    }

    /// Build without a shell, PATH lookup, inherited env, or hidden execution.
    /// Typed launch defaults stdin to null; explicit callers may configure pipes.
    /// Rechecking file types is not manifest trust or a race-free file identity grant.
    pub fn command(&self) -> Result<Command, ProgramError> {
        validate_paths(&self.executable, &self.working_directory)?;
        let mut command = Command::new(&self.executable);
        command
            .args(&self.arguments)
            .current_dir(&self.working_directory)
            .env_clear()
            .envs(self.environment.iter().map(|(name, value)| (name, value)))
            .stdin(Stdio::null());
        Ok(command)
    }

    /// Explicit internal JSON conversion, NOT a redacted diagnostic/log sink.
    /// Native non-Unicode paths/argv remain usable for launching, but fail here.
    pub fn to_json(&self) -> Result<serde_json::Value, ProgramError> {
        let arguments = self
            .arguments
            .iter()
            .map(|argument| json_string(argument, "argument"))
            .collect::<Result<Vec<_>, _>>()?;
        let environment = self
            .environment
            .iter()
            .map(|(name, value)| {
                Ok((
                    json_string(name, "environment name")?.to_owned(),
                    serde_json::Value::String(json_string(value, "environment value")?.to_owned()),
                ))
            })
            .collect::<Result<serde_json::Map<_, _>, ProgramError>>()?;
        Ok(serde_json::json!({
            "executable": json_string(self.executable.as_os_str(), "executable")?,
            "arguments": arguments,
            "working_directory": json_string(self.working_directory.as_os_str(), "working directory")?,
            "environment": environment,
        }))
    }
}

fn resolve_path(path: &Path, field: &'static str) -> Result<PathBuf, ProgramError> {
    reject_nul(path.as_os_str(), field)?;
    if !path.is_absolute() {
        return Err(ProgramError::RelativePath { field });
    }
    std::fs::canonicalize(path).map_err(|source| ProgramError::Path { field, source })
}

fn validate_paths(executable: &Path, directory: &Path) -> Result<(), ProgramError> {
    let metadata = std::fs::metadata(executable).map_err(|source| ProgramError::Path {
        field: "executable",
        source,
    })?;
    if !metadata.is_file() {
        return Err(ProgramError::NotExecutable);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(ProgramError::NotExecutable);
        }
    }
    #[cfg(windows)]
    {
        // Windows Command may implicitly use cmd.exe for batch files. Admit
        // native .exe inputs only, rather than claiming batch argv is shell-free.
        if !executable
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        {
            return Err(ProgramError::NotExecutable);
        }
    }
    if !std::fs::metadata(directory)
        .map_err(|source| ProgramError::Path {
            field: "working directory",
            source,
        })?
        .is_dir()
    {
        return Err(ProgramError::NotDirectory);
    }
    Ok(())
}

fn json_string<'a>(value: &'a OsStr, field: &'static str) -> Result<&'a str, ProgramError> {
    value
        .to_str()
        .ok_or(ProgramError::Unrepresentable { field })
}

fn reject_nul(value: &OsStr, field: &'static str) -> Result<(), ProgramError> {
    #[cfg(unix)]
    let contains_nul = {
        use std::os::unix::ffi::OsStrExt;
        value.as_bytes().contains(&0)
    };
    #[cfg(windows)]
    let contains_nul = {
        use std::os::windows::ffi::OsStrExt;
        value.encode_wide().any(|unit| unit == 0)
    };
    #[cfg(not(any(unix, windows)))]
    let contains_nul = match value.to_str() {
        Some(value) => value.contains('\0'),
        None => true,
    };
    if contains_nul {
        return Err(ProgramError::Nul { field });
    }
    Ok(())
}
