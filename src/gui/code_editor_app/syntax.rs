use std::ops::Range;

use crate::assembler::{DATA_STATEMENTS, INSTRUCTIONS, PSEUDO_INSTRUCTIONS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenKind {
    Instruction,
    Pseudo,
    Data,
    Comment,
}

pub(super) struct SyntaxHighlighter {
    current_line: usize,
}

impl iced::advanced::text::highlighter::Highlighter for SyntaxHighlighter {
    type Settings = ();
    type Highlight = TokenKind;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(_settings: &Self::Settings) -> Self {
        Self { current_line: 0 }
    }

    fn update(&mut self, _new_settings: &Self::Settings) {}

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        let _line_index = self.current_line;
        self.current_line = self.current_line.saturating_add(1);

        let mut highlights: Vec<(Range<usize>, TokenKind)> = Vec::new();
        let mut code = line;
        if let Some(comment_start) = line.find(';') {
            highlights.push((comment_start..line.len(), TokenKind::Comment));
            code = &line[..comment_start];
        }

        let mut it = code.char_indices().peekable();
        while let Some((start, ch)) = it.next() {
            if !ch.is_ascii_alphabetic() {
                continue;
            }

            let mut end = start + ch.len_utf8();
            while let Some(&(idx, next)) = it.peek() {
                if next.is_ascii_alphanumeric() {
                    it.next();
                    end = idx + next.len_utf8();
                } else {
                    break;
                }
            }

            if let Some(&(idx, ':')) = it.peek() {
                it.next();
                end = idx + 1;
                if let Some(&(idx2, ':')) = it.peek() {
                    it.next();
                    end = idx2 + 1;
                }
            }

            let token = &code[start..end];
            let kind = classify_token(token);
            if let Some(kind) = kind {
                highlights.push((start..end, kind));
            }
        }

        highlights.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

fn classify_token(token: &str) -> Option<TokenKind> {
    if INSTRUCTIONS.iter().any(|kw| token.eq_ignore_ascii_case(kw)) {
        Some(TokenKind::Instruction)
    } else if PSEUDO_INSTRUCTIONS
        .iter()
        .any(|kw| token.eq_ignore_ascii_case(kw))
    {
        Some(TokenKind::Pseudo)
    } else if DATA_STATEMENTS
        .iter()
        .any(|kw| token.eq_ignore_ascii_case(kw))
    {
        Some(TokenKind::Data)
    } else {
        None
    }
}
