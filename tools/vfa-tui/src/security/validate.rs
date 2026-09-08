use std::path::{Path, PathBuf};

use crate::error::TuiError;

/// Characters that are rejected in subprocess arguments.
const SHELL_METACHARACTERS: &[char] = &[
    ';', '|', '&', '$', '`', '\\', '<', '>', '(', ')', '{', '}', '!', '#', '*', '?', '[', ']',
    '\n', '\r', '\0',
];

/// Describes what class of invalid character was found in a registry path.
#[derive(Debug, PartialEq, Eq)]
pub enum InvalidCharClass {
    NullByte,
    ControlCharacter(u32),
    NonPrintableUnicode(u32),
}

impl std::fmt::Display for InvalidCharClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidCharClass::NullByte => write!(f, "null byte (U+0000)"),
            InvalidCharClass::ControlCharacter(cp) => write!(f, "control character (U+{cp:04X})"),
            InvalidCharClass::NonPrintableUnicode(cp) => {
                write!(f, "non-printable Unicode character (U+{cp:04X})")
            }
        }
    }
}

/// Validate a subprocess argument. Rejects if it contains any shell
/// metacharacters or C0/C1 control bytes (e.g. ESC 0x1B) — the metachar list
/// alone stops `; | & $` etc., but a raw control byte can still confuse a
/// terminal or downstream tool even when it can't be interpreted by a shell.
pub fn validate_argument(arg: &str) -> Result<(), TuiError> {
    for c in arg.chars() {
        if SHELL_METACHARACTERS.contains(&c) || crate::security::sanitize::is_disallowed_control(c)
        {
            return Err(TuiError::ValidationRejected {
                value: arg.to_string(),
                rule: format!("contains forbidden character {:?}", c),
            });
        }
    }
    Ok(())
}

/// Validate a model value for the model-policy builder.
///
/// Same denylist as [`validate_argument`] except that `[` and `]` are allowed,
/// because Cursor documents per-model parameters appended to the model id in
/// square brackets (`claude-opus-5[effort=high,context=300k]`). The relaxation
/// is deliberately confined to this one field rather than widened in
/// [`SHELL_METACHARACTERS`], so scope ids and every other argument keep the
/// stricter rule.
///
/// Brackets must form exactly one balanced group at the end of the value; a
/// stray, nested or leading bracket is rejected. Semantic validation of the
/// group's contents (documented keys, key=value shape, no duplicates) belongs
/// to `scripts/model-policy.mjs` and is not duplicated here.
pub fn validate_model_argument(arg: &str) -> Result<(), TuiError> {
    let reject = |rule: &str| {
        Err(TuiError::ValidationRejected {
            value: arg.to_string(),
            rule: rule.to_string(),
        })
    };
    for c in arg.chars() {
        if c == '[' || c == ']' {
            continue;
        }
        if SHELL_METACHARACTERS.contains(&c) || crate::security::sanitize::is_disallowed_control(c)
        {
            return Err(TuiError::ValidationRejected {
                value: arg.to_string(),
                rule: format!("contains forbidden character {c:?}"),
            });
        }
    }
    let opens = arg.matches('[').count();
    let closes = arg.matches(']').count();
    if opens == 0 && closes == 0 {
        return Ok(());
    }
    if opens != 1 || closes != 1 {
        return reject("model parameters must be a single \"[...]\" group");
    }
    let open_at = arg.find('[').expect("one open bracket");
    if !arg.ends_with(']') {
        return reject("model parameter group must be at the end of the value");
    }
    if open_at == 0 {
        return reject("model parameter group must follow a model id");
    }
    if arg.find(']').expect("one close bracket") < open_at {
        return reject("model parameter group brackets are inverted");
    }
    Ok(())
}

/// Validate a workspace registry path string.
///
/// Rejects paths that contain:
/// - Null bytes (U+0000) — Req 22.3, 20.5
/// - ASCII control characters (0x01-0x1F, 0x7F) — Req 22.3
/// - Unicode C1 controls (U+0080-U+009F) — Req 22.3
///
/// On rejection, the error message identifies the rejected path and the specific
/// invalid character class. Non-UTF-8 `OsStr` paths are also rejected.
pub fn validate_registry_path(path: &Path) -> Result<(), TuiError> {
    let path_str = path.to_str().ok_or_else(|| TuiError::ValidationRejected {
        value: path.to_string_lossy().to_string(),
        rule: "non-UTF-8 sequence in registry path".to_string(),
    })?;

    for c in path_str.chars() {
        let cp = c as u32;
        let invalid_class: Option<InvalidCharClass> = if cp == 0 {
            Some(InvalidCharClass::NullByte)
        } else if cp <= 0x1F || cp == 0x7F {
            Some(InvalidCharClass::ControlCharacter(cp))
        } else if (0x80..=0x9F).contains(&cp) {
            Some(InvalidCharClass::NonPrintableUnicode(cp))
        } else {
            None
        };

        if let Some(class) = invalid_class {
            return Err(TuiError::ValidationRejected {
                value: format!("registry path '{}' contains {}", path_str, class),
                rule: format!("registry_path_invalid_char: {class}"),
            });
        }
    }
    Ok(())
}

/// Validate a path is within the workspace root.
/// Canonicalizes both paths, then checks that the path starts with the workspace root.
/// Rejects null bytes and paths that escape the workspace.
pub fn validate_path(path: &Path, workspace_root: &Path) -> Result<PathBuf, TuiError> {
    // Reject null bytes in path
    if let Some(s) = path.to_str() {
        if s.contains('\0') {
            return Err(TuiError::PathTraversal {
                path: path.display().to_string(),
            });
        }
    } else {
        // Non-UTF-8 path - reject
        return Err(TuiError::PathTraversal {
            path: path.to_string_lossy().to_string(),
        });
    }

    let canonical_root = workspace_root
        .canonicalize()
        .map_err(|_| TuiError::PathTraversal {
            path: path.display().to_string(),
        })?;

    let canonical_path = path.canonicalize().map_err(|_| TuiError::PathTraversal {
        path: path.display().to_string(),
    })?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err(TuiError::PathTraversal {
            path: path.display().to_string(),
        });
    }

    Ok(canonical_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn validate_argument_accepts_safe() {
        assert!(validate_argument("hello-world_1.0/path").is_ok());
    }

    #[test]
    fn validate_model_argument_accepts_cursor_parameter_group() {
        assert!(validate_model_argument("composer-2.5").is_ok());
        assert!(validate_model_argument("claude-opus-5[effort=high]").is_ok());
        assert!(validate_model_argument("gpt-5.6-sol[effort=high,context=300k]").is_ok());
        // The bracket relaxation must not leak into the general validator.
        assert!(validate_argument("claude-opus-5[effort=high]").is_err());
    }

    #[test]
    fn validate_model_argument_still_rejects_metacharacters() {
        for bad in [
            "claude-opus-5[effort=high]; rm -rf /",
            "claude-opus-5[effort=$(id)]",
            "claude-opus-5[effort=`id`]",
            "claude-opus-5[effort=high]|tee",
        ] {
            assert!(validate_model_argument(bad).is_err(), "should reject {bad}");
        }
    }

    #[test]
    fn validate_model_argument_rejects_malformed_bracket_shapes() {
        for bad in [
            "claude-opus-5[effort=high",
            "claude-opus-5effort=high]",
            "[effort=high]",
            "claude[a=1][b=2]",
            "claude-opus-5[a=1]trailing",
            "claude-opus-5]a=1[",
        ] {
            assert!(validate_model_argument(bad).is_err(), "should reject {bad}");
        }
    }

    #[test]
    fn validate_argument_rejects_semicolon() {
        let err = validate_argument("cmd; rm -rf /").unwrap_err();
        match err {
            TuiError::ValidationRejected { rule, .. } => {
                assert!(rule.contains(';'));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_argument_rejects_pipe() {
        assert!(validate_argument("cmd | cat").is_err());
    }

    #[test]
    fn validate_argument_rejects_backtick() {
        assert!(validate_argument("$(whoami)").is_err());
    }

    #[test]
    fn validate_argument_rejects_null() {
        assert!(validate_argument("hello\0world").is_err());
    }

    #[test]
    fn validate_argument_rejects_newline() {
        assert!(validate_argument("line1\nline2").is_err());
    }

    #[test]
    fn validate_argument_rejects_escape_control_byte() {
        // SEC-7: ESC (0x1B) is a C0 control byte that predates the shell
        // metacharacter list — it must be rejected even though it's not one
        // of the punctuation characters in SHELL_METACHARACTERS.
        let err = validate_argument("gpt-5.6-sol\x1b[31m").unwrap_err();
        match err {
            TuiError::ValidationRejected { value, .. } => {
                assert!(value.contains('\x1b'));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn validate_argument_accepts_registered_model_names() {
        assert!(validate_argument("gpt-5.6-sol").is_ok());
        assert!(validate_argument("llama3.3:70b").is_ok());
        assert!(validate_argument("anthropic/claude-sonnet-4.5").is_ok());
    }

    #[test]
    fn validate_path_accepts_within_workspace() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();
        let file = sub.join("test.txt");
        fs::write(&file, "content").unwrap();

        let result = validate_path(&file, tmp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn validate_path_rejects_outside_workspace() {
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();
        let file = tmp2.path().join("secret.txt");
        fs::write(&file, "secret").unwrap();

        let result = validate_path(&file, tmp1.path());
        assert!(result.is_err());
    }

    #[test]
    fn validate_path_rejects_traversal() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("sub");
        fs::create_dir_all(&sub).unwrap();

        // Try to traverse out using ..
        let traversal = sub.join("..").join("..").join("etc").join("passwd");
        let result = validate_path(&traversal, tmp.path());
        assert!(result.is_err());
    }
}
