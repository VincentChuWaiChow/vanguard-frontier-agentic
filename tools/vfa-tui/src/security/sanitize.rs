/// Replace control bytes (0x00-0x08, 0x0B-0x0C, 0x0E-0x1F, 0x7F) with U+FFFD.
/// Preserves 0x09 (tab) and 0x0A (newline). All other characters unchanged.
pub fn sanitize_catalog_string(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if is_disallowed_control(c) {
                '\u{FFFD}'
            } else {
                c
            }
        })
        .collect()
}

/// Returns true if the input string contains any control bytes that would be sanitized.
pub fn has_control_bytes(input: &str) -> bool {
    input.chars().any(is_disallowed_control)
}

fn is_disallowed_control(c: char) -> bool {
    let b = c as u32;
    b <= 0x08
        || (0x0B..=0x0C).contains(&b)
        || (0x0E..=0x1F).contains(&b)
        || b == 0x7F
        || (0x80..=0x9F).contains(&b)
}

/// Pass through SGR sequences (ESC[ followed by numeric params separated by ; ending with 'm').
/// Strip all other ANSI escape sequences: OSC (ESC]), DCS (ESCP), SOS (ESCX),
/// PM (ESC^), APC (ESC_), and any other CSI sequences not ending in 'm'.
/// Preserve all non-escape content.
pub fn sanitize_subprocess_output(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == 0x1B {
            // ESC found
            if i + 1 < len {
                match bytes[i + 1] {
                    b'[' => {
                        // CSI sequence - parse parameters
                        let start = i;
                        i += 2; // skip ESC[
                                // Collect parameter bytes (0x30-0x3F) and intermediate (0x20-0x2F)
                        while i < len && bytes[i] >= 0x20 && bytes[i] <= 0x3F {
                            i += 1;
                        }
                        // Final byte (0x40-0x7E)
                        if i < len && bytes[i] >= 0x40 && bytes[i] <= 0x7E {
                            let final_byte = bytes[i];
                            i += 1;
                            if final_byte == b'm' {
                                // SGR sequence - keep it
                                result.push_str(&input[start..i]);
                            }
                            // else: non-SGR CSI sequence - strip it
                        }
                        // else: malformed - just skip what we consumed
                    }
                    b']' => {
                        // OSC sequence - skip until ST (ESC\ or BEL)
                        i += 2;
                        while i < len {
                            if bytes[i] == 0x07 {
                                i += 1;
                                break;
                            }
                            if bytes[i] == 0x1B && i + 1 < len && bytes[i + 1] == b'\\' {
                                i += 2;
                                break;
                            }
                            i += 1;
                        }
                    }
                    b'P' | b'X' | b'^' | b'_' => {
                        // DCS, SOS, PM, APC - skip until ST (ESC\ or BEL)
                        i += 2;
                        while i < len {
                            if bytes[i] == 0x1B && i + 1 < len && bytes[i + 1] == b'\\' {
                                i += 2;
                                break;
                            }
                            if bytes[i] == 0x07 {
                                i += 1;
                                break;
                            }
                            i += 1;
                        }
                    }
                    _ => {
                        // Other ESC sequences (e.g., ESC( ESC) etc.) - strip 2 bytes
                        i += 2;
                    }
                }
            } else {
                // Lone ESC at end - skip it
                i += 1;
            }
        } else {
            // Regular character - keep it
            // Handle multi-byte UTF-8 properly
            let c = input[i..].chars().next().unwrap();
            if !is_disallowed_control(c) {
                result.push(c);
            }
            i += c.len_utf8();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_preserves_normal_text() {
        assert_eq!(sanitize_catalog_string("hello world"), "hello world");
    }

    #[test]
    fn sanitize_preserves_tab_and_newline() {
        assert_eq!(sanitize_catalog_string("a\tb\nc"), "a\tb\nc");
    }

    #[test]
    fn sanitize_replaces_null() {
        assert_eq!(sanitize_catalog_string("a\x00b"), "a\u{FFFD}b");
    }

    #[test]
    fn sanitize_replaces_control_range() {
        // 0x01 through 0x08
        let input = "a\x01\x02\x03\x04\x05\x06\x07\x08b";
        let result = sanitize_catalog_string(input);
        assert_eq!(
            result,
            "a\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}b"
        );
    }

    #[test]
    fn sanitize_replaces_0b_0c() {
        assert_eq!(sanitize_catalog_string("a\x0B\x0Cb"), "a\u{FFFD}\u{FFFD}b");
    }

    #[test]
    fn sanitize_replaces_0e_through_1f() {
        let input = "\x0E\x0F\x10\x1F";
        let result = sanitize_catalog_string(input);
        assert_eq!(result, "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}");
    }

    #[test]
    fn sanitize_replaces_del() {
        assert_eq!(sanitize_catalog_string("a\x7Fb"), "a\u{FFFD}b");
    }

    #[test]
    fn sanitize_replaces_c1_controls() {
        assert_eq!(sanitize_catalog_string("a\u{009B}b"), "a\u{FFFD}b");
    }

    #[test]
    fn has_control_bytes_detects_null() {
        assert!(has_control_bytes("hello\x00world"));
    }

    #[test]
    fn has_control_bytes_clean_string() {
        assert!(!has_control_bytes("hello\tworld\n"));
    }

    #[test]
    fn subprocess_preserves_sgr() {
        let input = "\x1B[31mred\x1B[0m normal";
        let result = sanitize_subprocess_output(input);
        assert_eq!(result, "\x1B[31mred\x1B[0m normal");
    }

    #[test]
    fn subprocess_preserves_complex_sgr() {
        let input = "\x1B[1;32;48;5;196mtext\x1B[0m";
        let result = sanitize_subprocess_output(input);
        assert_eq!(result, "\x1B[1;32;48;5;196mtext\x1B[0m");
    }

    #[test]
    fn subprocess_strips_cursor_movement() {
        let input = "before\x1B[2Aafter";
        let result = sanitize_subprocess_output(input);
        assert_eq!(result, "beforeafter");
    }

    #[test]
    fn subprocess_strips_osc() {
        let input = "before\x1B]0;title\x07after";
        let result = sanitize_subprocess_output(input);
        assert_eq!(result, "beforeafter");
    }

    #[test]
    fn subprocess_strips_dcs() {
        let input = "before\x1BPdata\x1B\\after";
        let result = sanitize_subprocess_output(input);
        assert_eq!(result, "beforeafter");
    }

    #[test]
    fn subprocess_strips_apc() {
        let input = "before\x1B_data\x1B\\after";
        let result = sanitize_subprocess_output(input);
        assert_eq!(result, "beforeafter");
    }

    #[test]
    fn subprocess_strips_c1_controls() {
        let input = "before\u{009B}31mafter";
        let result = sanitize_subprocess_output(input);
        assert_eq!(result, "before31mafter");
    }

    #[test]
    fn subprocess_preserves_plain_text() {
        let input = "hello world\nline 2";
        assert_eq!(sanitize_subprocess_output(input), input);
    }
}

// Fixed-state sanitizer for safe streaming display. Unlike the legacy API,
// strips even SGR and removes controls before credential recognition.
#[derive(Default)]
pub(crate) struct StreamingSanitizer {
    escape: u8,
    utf8: [u8; 4],
    len: usize,
    needed: usize,
}
impl StreamingSanitizer {
    pub(crate) fn push(&mut self, byte: u8, emit: &mut impl FnMut(&[u8])) {
        match self.escape {
            1 => {
                self.escape = match byte {
                    b'[' => 2,
                    b']' | b'P' | b'X' | b'^' | b'_' => 3,
                    _ => 0,
                };
                return;
            }
            2 => {
                if (0x40..=0x7e).contains(&byte) || !(0x20..=0x3f).contains(&byte) {
                    self.escape = 0;
                }
                return;
            }
            3 => {
                if byte == 7 {
                    self.escape = 0;
                } else if byte == 0x1b {
                    self.escape = 4;
                }
                return;
            }
            4 => {
                self.escape = if byte == b'\\' || byte == 7 {
                    0
                } else if byte == 0x1b {
                    4
                } else {
                    3
                };
                return;
            }
            _ => {}
        }
        if byte == 0x1b {
            self.escape = 1;
            return;
        }
        // An inserted NUL/CR cannot create an artificial key delimiter.
        if byte < 0x20 && byte != b'\n' && byte != b'\t' || byte == 0x7f {
            return;
        }
        if self.len > 0 {
            if (0x80..=0xbf).contains(&byte) {
                self.utf8[self.len] = byte;
                self.len += 1;
                if self.len == self.needed {
                    match std::str::from_utf8(&self.utf8[..self.len]) {
                        Ok(s) if !s.chars().any(is_disallowed_control) => emit(s.as_bytes()),
                        Ok(_) => {}
                        Err(_) => emit("�".as_bytes()),
                    }
                    self.len = 0;
                }
                return;
            }
            self.len = 0;
            emit("�".as_bytes());
        }
        if byte.is_ascii() {
            emit(&[byte]);
        } else {
            self.needed = match byte {
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => {
                    emit("�".as_bytes());
                    return;
                }
            };
            self.utf8[0] = byte;
            self.len = 1;
        }
    }
    pub(crate) fn finish(&mut self, emit: &mut impl FnMut(&[u8])) {
        if self.len > 0 {
            emit("�".as_bytes());
        }
        self.len = 0;
        self.escape = 0;
    }
}
