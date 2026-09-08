use std::ffi::OsString;

/// Check if an environment variable name indicates it contains a secret value.
/// Case-insensitive match for exact names and substring patterns.
/// For `_KEY`, only matches when the name ends with `_KEY` or contains `_KEY_`
/// to avoid false positives on names like `KEYBOARD_LAYOUT` or `KEYRING_CONTROL`.
pub fn is_secret_env_var(name: &str) -> bool {
    let upper = name.to_uppercase();

    // Exact matches
    if upper == "AWS_SECRET_ACCESS_KEY"
        || upper == "GITHUB_TOKEN"
        || upper == "NPM_TOKEN"
        || upper == "SECRET"
        || upper == "TOKEN"
        || upper == "PASSWORD"
        || upper == "CREDENTIAL"
        || upper == "KEY"
        || upper == "SSH_AUTH_SOCK"
        || upper == "SSH_AGENT_PID"
        || upper == "GIT_ASKPASS"
        || upper == "GIT_ASKPASS_REQUIRE"
        || upper == "GIT_SSH"
        || upper == "GIT_SSH_COMMAND"
        || upper == "KUBECONFIG"
        || upper == "AWS_PROFILE"
        || upper == "AWS_CONFIG_FILE"
        || upper == "AWS_SHARED_CREDENTIALS_FILE"
        || upper == "GOOGLE_APPLICATION_CREDENTIALS"
        || upper == "AZURE_CONFIG_DIR"
        || upper == "NODE_OPTIONS"
    {
        return true;
    }

    // Substring matches (these are unambiguous)
    if upper.contains("_SECRET")
        || upper.contains("_TOKEN")
        || upper.contains("_PASSWORD")
        || upper.contains("_CREDENTIAL")
        || upper.starts_with("NPM_CONFIG_")
    {
        return true;
    }

    // _KEY: only match as suffix or when followed by _ (e.g., _KEY_ID)
    // This catches API_KEY, SECRET_KEY, ACCESS_KEY_ID but not KEYBOARD, KEYRING
    if upper.ends_with("_KEY") || upper.contains("_KEY_") {
        return true;
    }

    false
}

/// Redact known secret patterns in a string, replacing them with `[REDACTED]`.
///
/// Patterns matched:
/// - `ghp_` followed by 36+ alphanumeric characters (GitHub PAT)
/// - `github_pat_` followed by 22+ token characters (GitHub fine-grained PAT)
/// - `npm_` followed by 36+ characters (npm token)
/// - `sk-` followed by 20+ characters (API key)
/// - `xoxb-` or `xoxp-` followed by 20+ characters (Slack tokens)
/// - `AKIA` followed by 16+ uppercase alphanumeric characters (AWS access key ID)
/// - JWT-shaped strings with three base64url segments
/// - PEM private key blocks
/// - Base64-like strings >40 contiguous characters from `[A-Za-z0-9+/=]`
pub fn redact_secrets(input: &str) -> String {
    let (detection_input, index_map) = detection_view_without_ansi(input);

    // First pass: identify spans to redact (start, end) using pattern matching.
    let mut redactions: Vec<(usize, usize)> = Vec::new();

    // Find ghp_ tokens
    find_prefixed_spans(&detection_input, "ghp_", 36, is_alnum, &mut redactions);
    // Find GitHub fine-grained tokens
    find_prefixed_spans(
        &detection_input,
        "github_pat_",
        22,
        is_github_pat_char,
        &mut redactions,
    );
    // Find npm_ tokens
    find_prefixed_spans(
        &detection_input,
        "npm_",
        36,
        is_npm_token_char,
        &mut redactions,
    );
    // Find sk- keys
    find_prefixed_spans(&detection_input, "sk-", 20, is_sk_char, &mut redactions);
    // Find Slack-style tokens
    find_prefixed_spans(&detection_input, "xoxb-", 20, is_sk_char, &mut redactions);
    find_prefixed_spans(&detection_input, "xoxp-", 20, is_sk_char, &mut redactions);
    // Find AKIA keys
    find_akia_spans(&detection_input, &mut redactions);
    // Find JWT-shaped strings
    find_jwt_spans(&detection_input, &mut redactions);
    // Find PEM private key blocks
    find_private_key_spans(&detection_input, &mut redactions);
    // Find base64 spans >40 chars
    find_base64_spans(&detection_input, &mut redactions);

    if redactions.is_empty() {
        return input.to_string();
    }

    let mut redactions = redactions
        .into_iter()
        .filter_map(|(start, end)| {
            if start >= end || end > index_map.len() {
                return None;
            }
            Some((index_map[start], index_map[end - 1] + 1))
        })
        .collect::<Vec<_>>();

    // Sort by start position and merge overlapping spans
    redactions.sort_by_key(|&(start, _)| start);
    let merged = merge_spans(&redactions);

    // Build result
    let mut result = String::with_capacity(input.len());
    let mut pos = 0;
    for (start, end) in merged {
        if pos < start {
            result.push_str(&input[pos..start]);
        }
        result.push_str("[REDACTED]");
        pos = end;
    }
    if pos < input.len() {
        result.push_str(&input[pos..]);
    }

    result
}

fn detection_view_without_ansi(input: &str) -> (String, Vec<usize>) {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index_map = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B {
            i = skip_escape_sequence(bytes, i);
            continue;
        }
        output.push(bytes[i]);
        index_map.push(i);
        i += 1;
    }
    (
        String::from_utf8(output).unwrap_or_else(|_| input.to_string()),
        index_map,
    )
}

fn skip_escape_sequence(bytes: &[u8], start: usize) -> usize {
    if start + 1 >= bytes.len() {
        return start + 1;
    }
    match bytes[start + 1] {
        b'[' => {
            let mut i = start + 2;
            while i < bytes.len() && (0x20..=0x3F).contains(&bytes[i]) {
                i += 1;
            }
            if i < bytes.len() && (0x40..=0x7E).contains(&bytes[i]) {
                i + 1
            } else {
                i
            }
        }
        b']' | b'P' | b'X' | b'^' | b'_' => {
            let mut i = start + 2;
            while i < bytes.len() {
                if bytes[i] == 0x07 {
                    return i + 1;
                }
                if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                    return i + 2;
                }
                i += 1;
            }
            i
        }
        _ => start + 2,
    }
}

fn is_alnum(c: u8) -> bool {
    c.is_ascii_alphanumeric()
}

fn is_npm_token_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'-'
}

fn is_github_pat_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

fn is_sk_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'-' || c == b'_'
}

fn is_base64url_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'-' || c == b'_'
}

fn is_base64_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'+' || c == b'/' || c == b'='
}

fn find_prefixed_spans(
    input: &str,
    prefix: &str,
    min_suffix_len: usize,
    is_valid: fn(u8) -> bool,
    spans: &mut Vec<(usize, usize)>,
) {
    let bytes = input.as_bytes();
    let prefix_bytes = prefix.as_bytes();
    let prefix_len = prefix_bytes.len();

    let mut start = 0;
    while let Some(pos) = find_substr(bytes, prefix_bytes, start) {
        let suffix_start = pos + prefix_len;
        let mut end = suffix_start;
        while end < bytes.len() && is_valid(bytes[end]) {
            end += 1;
        }
        if end - suffix_start >= min_suffix_len {
            spans.push((pos, end));
        }
        start = pos + 1;
    }
}

fn find_akia_spans(input: &str, spans: &mut Vec<(usize, usize)>) {
    let bytes = input.as_bytes();
    let prefix = b"AKIA";

    let mut start = 0;
    while let Some(pos) = find_substr(bytes, prefix, start) {
        let suffix_start = pos + 4;
        let mut end = suffix_start;
        while end < bytes.len() && (bytes[end].is_ascii_uppercase() || bytes[end].is_ascii_digit())
        {
            end += 1;
        }
        if end - suffix_start >= 16 {
            spans.push((pos, end));
        }
        start = pos + 1;
    }
}

fn find_jwt_spans(input: &str, spans: &mut Vec<(usize, usize)>) {
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && !is_base64url_byte(bytes[i]) {
            i += 1;
        }
        let start = i;
        let mut dots = Vec::new();
        while i < bytes.len() && (is_base64url_byte(bytes[i]) || bytes[i] == b'.') {
            if bytes[i] == b'.' {
                dots.push(i);
            }
            i += 1;
        }
        if dots.len() == 2 {
            let first = dots[0] - start;
            let second = dots[1] - dots[0] - 1;
            let third = i - dots[1] - 1;
            if first >= 10 && second >= 10 && third >= 10 {
                spans.push((start, i));
            }
        }
    }
}

fn find_private_key_spans(input: &str, spans: &mut Vec<(usize, usize)>) {
    let mut start = 0;
    while let Some(begin_rel) = input[start..].find("-----BEGIN ") {
        let begin = start + begin_rel;
        let Some(header_end_rel) = input[begin..].find("PRIVATE KEY-----") else {
            start = begin + 1;
            continue;
        };
        let body_start = begin + header_end_rel + "PRIVATE KEY-----".len();
        let Some(end_rel) = input[body_start..].find("-----END ") else {
            start = body_start;
            continue;
        };
        let end_start = body_start + end_rel;
        let Some(end_suffix_rel) = input[end_start..].find("PRIVATE KEY-----") else {
            start = end_start + 1;
            continue;
        };
        let end = end_start + end_suffix_rel + "PRIVATE KEY-----".len();
        spans.push((begin, end));
        start = end;
    }
}

fn find_base64_spans(input: &str, spans: &mut Vec<(usize, usize)>) {
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if is_base64_byte(bytes[i]) {
            let start = i;
            let mut has_base64_special = false;
            while i < bytes.len() && is_base64_byte(bytes[i]) {
                if bytes[i] == b'+' || bytes[i] == b'/' || bytes[i] == b'=' {
                    has_base64_special = true;
                }
                i += 1;
            }
            // Only treat as base64 if >40 chars AND contains at least one +, /, or =
            // This avoids false positives on pure hex strings and plain alphanumeric text
            if i - start > 40 && has_base64_special {
                spans.push((start, i));
            }
        } else {
            i += 1;
        }
    }
}

fn find_substr(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start + needle.len() > haystack.len() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + start)
}

fn merge_spans(spans: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if spans.is_empty() {
        return Vec::new();
    }
    let mut merged = vec![spans[0]];
    for &(start, end) in &spans[1..] {
        let last = merged.last_mut().unwrap();
        if start <= last.1 {
            last.1 = last.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    merged
}

/// Collect environment variables, filtering out entries where the key is identified as a secret.
pub fn sanitized_child_env() -> Vec<(OsString, OsString)> {
    std::env::vars_os()
        .filter(|(key, _)| {
            if let Some(name) = key.to_str() {
                !is_secret_env_var(name)
            } else {
                // Drop names we cannot classify safely.
                false
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_secret_exact_matches() {
        assert!(is_secret_env_var("AWS_SECRET_ACCESS_KEY"));
        assert!(is_secret_env_var("GITHUB_TOKEN"));
        assert!(is_secret_env_var("NPM_TOKEN"));
    }

    #[test]
    fn is_secret_case_insensitive() {
        assert!(is_secret_env_var("aws_secret_access_key"));
        assert!(is_secret_env_var("github_token"));
        assert!(is_secret_env_var("Npm_Token"));
    }

    #[test]
    fn is_secret_substring_matches() {
        assert!(is_secret_env_var("MY_SECRET_VALUE"));
        assert!(is_secret_env_var("TOKEN"));
        assert!(is_secret_env_var("PASSWORD"));
        assert!(is_secret_env_var("API_KEY"));
        assert!(is_secret_env_var("AUTH_TOKEN"));
        assert!(is_secret_env_var("DB_PASSWORD"));
        assert!(is_secret_env_var("CLOUD_CREDENTIAL"));
        assert!(is_secret_env_var("ACCESS_KEY_ID"));
        assert!(is_secret_env_var("SECRET_KEY"));
    }

    #[test]
    fn is_secret_credential_delegation_env() {
        assert!(is_secret_env_var("SSH_AUTH_SOCK"));
        assert!(is_secret_env_var("GIT_ASKPASS"));
        assert!(is_secret_env_var("GIT_SSH_COMMAND"));
        assert!(is_secret_env_var("KUBECONFIG"));
        assert!(is_secret_env_var("AWS_PROFILE"));
        assert!(is_secret_env_var("GOOGLE_APPLICATION_CREDENTIALS"));
        assert!(is_secret_env_var("AZURE_CONFIG_DIR"));
        assert!(is_secret_env_var("NODE_OPTIONS"));
        assert!(is_secret_env_var("NPM_CONFIG_USERCONFIG"));
    }

    #[test]
    fn is_secret_non_secret() {
        assert!(!is_secret_env_var("PATH"));
        assert!(!is_secret_env_var("HOME"));
        assert!(!is_secret_env_var("LANG"));
        assert!(!is_secret_env_var("SHELL"));
        assert!(!is_secret_env_var("KEYBOARD_LAYOUT"));
        assert!(!is_secret_env_var("GNOME_KEYRING_CONTROL"));
        assert!(!is_secret_env_var("XKB_DEFAULT_LAYOUT"));
    }

    #[test]
    fn redact_github_pat() {
        let token = ["gh", "p_", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "abcdefghijklmn"].concat();
        let result = redact_secrets(&format!("token: {token}"));
        assert_eq!(result, "token: [REDACTED]");
    }

    #[test]
    fn redact_npm_token() {
        let token = ["np", "m_", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "abcdefghijklmn"].concat();
        let result = redact_secrets(&format!("npm: {token}"));
        assert_eq!(result, "npm: [REDACTED]");
    }

    #[test]
    fn redact_github_fine_grained_pat() {
        let token = ["github", "_pat_", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "_123456"].concat();
        let result = redact_secrets(&format!("token: {token}"));
        assert_eq!(result, "token: [REDACTED]");
    }

    #[test]
    fn redact_sk_key() {
        let key = ["s", "k-", "abcdefghijklmnopqrstuvwxyz"].concat();
        let result = redact_secrets(&format!("key: {key}"));
        assert_eq!(result, "key: [REDACTED]");
    }

    #[test]
    fn redact_aws_key_id() {
        let key = ["AK", "IAIOSFODNN7EXAMPLE1"].concat();
        let result = redact_secrets(&format!("aws: {key}"));
        assert_eq!(result, "aws: [REDACTED]");
    }

    #[test]
    fn redact_jwt() {
        let jwt = [
            "eyJhbGciOiJIUzI1Ni",
            "J9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.",
            "SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
        ]
        .concat();
        let result = redact_secrets(&format!("bearer {jwt}"));
        assert_eq!(result, "bearer [REDACTED]");
    }

    #[test]
    fn redact_private_key_block() {
        let key = [
            "-----BEGIN PRIVATE ",
            "KEY-----\nabc\n-----END PRIVATE KEY-----",
        ]
        .concat();
        let result = redact_secrets(&format!("key={key}"));
        assert_eq!(result, "key=[REDACTED]");
    }

    #[test]
    fn redact_base64_long() {
        let b64 = ["ABCDEFGHIJKLMNOPQRST", "UVWXYZabcdefghijk+mnopqrs="].concat();
        assert!(b64.len() > 40);
        let result = redact_secrets(&format!("data: {b64} end"));
        assert_eq!(result, "data: [REDACTED] end");
    }

    #[test]
    fn redact_preserves_short_strings() {
        let input = "hello world PATH=/usr/bin";
        assert_eq!(redact_secrets(input), input);
    }

    #[test]
    fn redact_preserves_normal_text() {
        let input = "This is a normal log message with no secrets.";
        assert_eq!(redact_secrets(input), input);
    }

    #[test]
    fn redact_preserves_pure_hex_string() {
        // SHA-256 hex digests should NOT be redacted (no +, /, or = characters)
        let hex = [
            "e3b0c44298fc1c149afbf4c8996fb924",
            "27ae41e4649b934ca495991b7852b855",
        ]
        .concat();
        assert!(hex.len() > 40);
        let result = redact_secrets(&format!("hash: {hex}"));
        assert_eq!(result, format!("hash: {hex}"));
    }

    #[test]
    fn redact_preserves_long_alphanumeric() {
        // Pure alphanumeric strings >40 chars without base64 specials should not be redacted
        let long_str = ["ABCDEFGHIJKLMNOPQRST", "UVWXYZabcdefghijklmnopqrs"].concat();
        assert!(long_str.len() > 40);
        let result = redact_secrets(&format!("data: {long_str} end"));
        assert_eq!(result, format!("data: {long_str} end"));
    }

    #[test]
    fn sanitized_env_filters_secrets() {
        // This test just verifies the function runs without panic.
        // Actual filtering depends on the running environment.
        let env = sanitized_child_env();
        for (key, _) in &env {
            if let Some(name) = key.to_str() {
                assert!(
                    !is_secret_env_var(name),
                    "secret env var {name} was not filtered"
                );
            }
        }
    }

    #[test]
    fn redact_secrets_after_ansi_strip() {
        use crate::security::sanitize::sanitize_subprocess_output;
        // A GitHub PAT token with ANSI color codes inserted in the middle
        // After sanitize strips the ANSI, the full token pattern should be visible and redacted
        let token = ["gh", "p_", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "abcdefghijklmn"].concat();
        let with_ansi = format!("secret: \x1B[31m{}\x1B[0m end", token);
        let sanitized = sanitize_subprocess_output(&with_ansi);
        let redacted = redact_secrets(&sanitized);
        assert!(
            !redacted.contains("ghp_"),
            "Token should be redacted but found: {redacted}"
        );
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn redact_secret_split_by_sgr_sequence() {
        let input = [
            "gh",
            "p_1234567890abcdefghij",
            "\x1B[31m",
            "klmnopqrstuvwxyz",
        ]
        .concat();
        assert_eq!(redact_secrets(&input), "[REDACTED]");
    }
}
