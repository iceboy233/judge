use std::io::{self, BufReader, Read};

pub trait Comparator {
    fn compare(&self, answer: &mut impl Read, output: &mut impl Read) -> io::Result<CompareResult>;
}

pub struct CompareResult {
    pub ok: bool,
}

pub struct TokenComparator;

impl Comparator for TokenComparator {
    fn compare(&self, answer: &mut impl Read, output: &mut impl Read) -> io::Result<CompareResult> {
        let mut answer_bytes = BufReader::new(answer).bytes();
        let mut output_bytes = BufReader::new(output).bytes();
        let mut allow_space = true;
        let mut allow_newline = true;
        let mut a;
        let mut b;
        loop {
            a = answer_bytes.next().transpose()?;
            b = output_bytes.next().transpose()?;
            while a != b && !(is_space(a) && is_space(b)) {
                if (is_space(a) && (allow_space || is_newline(b) || b.is_none()))
                    || (is_newline(a) && (allow_newline || b.is_none()))
                    || matches!(a, Some(b'\r'))
                {
                    a = answer_bytes.next().transpose()?;
                } else if (is_space(b) && (allow_space || is_newline(a) || a.is_none()))
                    || (is_newline(b) && (allow_newline || a.is_none()))
                    || matches!(b, Some(b'\r'))
                {
                    b = output_bytes.next().transpose()?;
                } else {
                    return Ok(CompareResult { ok: false });
                }
            }
            if a.is_none() {
                return Ok(CompareResult { ok: true });
            }
            allow_newline = is_newline(a);
            allow_space = allow_newline || is_space(a);
        }
    }
}

fn is_space(a: Option<u8>) -> bool {
    matches!(a, Some(b' ' | b'\t'))
}

fn is_newline(a: Option<u8>) -> bool {
    matches!(a, Some(b'\n'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare(a: &str, b: &str) -> bool {
        let ok1 = TokenComparator
            .compare(&mut a.as_bytes(), &mut b.as_bytes())
            .unwrap()
            .ok;
        if a != b {
            let ok2 = TokenComparator
                .compare(&mut b.as_bytes(), &mut a.as_bytes())
                .unwrap()
                .ok;
            assert_eq!(ok1, ok2);
        }
        ok1
    }

    #[test]
    fn test_token_comparator() {
        // Exact matches.
        assert!(compare("", ""));
        assert!(compare("1 2 3", "1 2 3"));
        assert!(compare("nana", "nana"));
        assert!(!compare("na", "nana"));
        assert!(!compare("nana", "banana"));

        // Whitespaces and newlines.
        assert!(compare("", " "));
        assert!(compare("", "\n"));
        assert!(compare("", "\n "));
        assert!(compare("", " \n"));
        assert!(compare(" ", "\n"));
        assert!(compare("1\t2", "1 2"));
        assert!(compare("1\n\n2", "1\n2"));
        assert!(compare("1 \n2", "1\n2"));
        assert!(compare("1\n 2", "1\n2"));
        assert!(compare("1 2\t\t3", "1 2 3"));
        assert!(compare("1 2 3", "1\t2   3"));
        assert!(!compare("1 23", "1 2 3"));
        assert!(compare("line1  \nline2", "line1\nline2"));

        // Cross platform newlines.
        assert!(compare("1 2 3\r\n", "1 2 3\n"));
        assert!(compare("line1\r\nline2", "line1\nline2"));
        assert!(!compare("line1\rline2", "line1\nline2"));
        assert!(!compare("line1\r\nline2", "line1\rline2"));
        assert!(!compare("line1\r\r\nline2", "line1\rline2"));

        // Leading and trailing whitespaces.
        assert!(compare("1\n", "1"));
        assert!(compare("1 ", "1"));
        assert!(compare("1\t", "1"));
        assert!(compare("1\n ", "1 \n"));
        assert!(compare("  1 2 3", "1 2 3"));
        assert!(compare("\n\n1 2 3", "1 2 3"));
        assert!(compare("\n \n1 2 3", "1 2 3"));
        assert!(compare("1 2 3\n\n  \n", "1 2 3"));

        // Wrong answers.
        assert!(!compare("", "1"));
        assert!(!compare("1", "2"));
        assert!(!compare("1", "1 2"));
        assert!(!compare("1 2", "12"));
        assert!(!compare("1 2 3", "1 2 4"));
        assert!(!compare("1 2", "1\n2"));
        assert!(!compare("1  2", "1\n 2"));
        assert!(!compare("1  2", "1 \n2"));
        assert!(!compare("1  2", "1\t\n 2"));
    }
}
