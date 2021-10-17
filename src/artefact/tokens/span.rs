use std::{fmt::Display, ops::Range, path::PathBuf};

use backtrace::Backtrace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span(Range<usize>);

impl Span {
    pub fn new(range: Range<usize>) -> Self {
        Self { 0: range }
    }

    pub fn make_error_span(&self, code: &Vec<char>, source_type: SourceType) -> ErrorSpan {
        let mut range = self.range();
        range.start = range.start.min(code.len());
        range.end = range.end.min(code.len());

        log::debug!("backtrace: {:?}", Backtrace::new());

        let before = code[..range.start]
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, c)| (range.start - i, c))
            .find(|&(_, &c)| c == '\n')
            .map(|(i, _)| i);

        let after = code[range.end..]
            .into_iter()
            .enumerate()
            .map(|(i, c)| (range.end + i, c))
            .find(|&(_, &c)| c == '\n')
            .map(|(i, _)| i);

        let line_span = match (before, after) {
            (Some(before), Some(after)) => Span::new(before..after),
            (None, Some(after)) => Span::new(0..after),
            (Some(before), None) => Span::new(before..code.len()),
            (None, None) => Span::new(0..code.len()),
        };

        let mut row = 0;
        let mut col = 0;
        for c in code[..range.start].into_iter() {
            match c {
                '\n' => {
                    col = 0;
                    row += 1
                }
                _ => col += 1,
            }
        }

        let code_row = code[line_span.range()].into_iter().collect();

        ErrorSpan {
            error_span: self.clone(),
            line_span,

            row,
            col,

            code_row,
            source_type,
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    Stdin,
    File(PathBuf),
}

impl Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Stdin => write!(f, "<stdin>"),
            SourceType::File(path) => write!(f, "{}", path.as_os_str().to_string_lossy()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorSpan {
    error_span: Span,
    line_span: Span,

    row: usize,
    col: usize,

    code_row: String,
    source_type: SourceType,
}

impl Display for ErrorSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_range = self.error_span.range();

        write!(
            f,
            "  at {}:{}:{}\n\n  {}\n  {}{} ",
            self.source_type,
            self.row + 1,
            self.col + 1,
            self.code_row,
            " ".repeat(self.col),
            "^".repeat(error_range.len())
        )
    }
}
