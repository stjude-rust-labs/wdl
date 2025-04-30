//! Utility functions for wdl-analysis.

// Add any additional imports needed

/// Iterates over the lines of a string and returns the line, starting offset,
/// and next possible starting offset.
pub fn lines_with_offset(s: &str) -> impl Iterator<Item = (&str, usize, usize)> {
    let mut offset = 0;
    std::iter::from_fn(move || {
        if offset >= s.len() {
            return None;
        }

        let start = offset;
        loop {
            match s[offset..].find(|c| ['\r', '\n'].contains(&c)) {
                Some(i) => {
                    let end = offset + i;
                    offset = end + 1;

                    if s.as_bytes().get(end) == Some(&b'\r') {
                        if s.as_bytes().get(end + 1) != Some(&b'\n') {
                            continue;
                        }

                        // There are two characters in the newline
                        offset += 1;
                    }

                    return Some((&s[start..end], start, offset));
                }
                None => {
                    offset = s.len();
                    return Some((&s[start..], start, offset));
                }
            }
        }
    })
}
