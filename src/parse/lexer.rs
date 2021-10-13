use unicode_segmentation::UnicodeSegmentation;

use crate::artefact::tokens::Tokens;

pub fn run_lexer(code: &str) -> Tokens<'_> {
    let mut in_string = None;

    code.graphemes(true).filter_map(|c| {
        if let Some(string) = in_string {}

        match (c, in_string) {
            ("\"", None) => in_string = Some(String::new()),
        };
        None
    });

    todo!()
}
