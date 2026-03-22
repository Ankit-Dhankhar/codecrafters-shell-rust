use std::borrow::Cow;

use rustyline::{
    Context, Helper,
    completion::{Completer, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};

use crate::builtins::BUILTINS;
use crate::filename_completer::complete_filename;
use crate::trie::Trie;
use crate::utils::get_all_executable_paths;

pub struct ShellCompleter {
    trie: Trie,
}

impl ShellCompleter {
    pub fn new() -> Self {
        let mut trie = Trie::new();

        for cmd in BUILTINS {
            trie.insert(cmd);
        }

        for cmd in get_all_executable_paths() {
            trie.insert(&cmd);
        }

        ShellCompleter { trie }
    }
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let word_start = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let prefix = &line[word_start..pos];

        // Only complete at command position (first word)
        if word_start == 0 && !prefix.is_empty() {
            let mut matches: Vec<Pair> = self
                .trie
                .find_with_prefix(prefix)
                .into_iter()
                .filter(|word| word != prefix)
                .map(|word| Pair {
                    display: word.clone(),
                    replacement: format!("{} ", word),
                })
                .collect();

            matches.sort_by(|a, b| a.display.cmp(&b.display));
            matches.dedup_by(|a, b| a.display == b.display);

            return Ok((word_start, matches));
        }

        let matches = complete_filename(prefix);

        Ok((word_start, matches))
    }
}

impl Hinter for ShellCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        for i in (0..ctx.history().len()).rev() {
            if let Ok(Some(search_result)) = ctx
                .history()
                .get(i, rustyline::history::SearchDirection::Reverse)
            {
                let entry = search_result.entry;
                if entry.starts_with(line) && entry.len() > line.len() {
                    return Some(entry[line.len()..].to_string());
                }
            }
        }
        None
    }
}

impl Highlighter for ShellCompleter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        Cow::Owned(format!("\x1b[32m{}\x1b[0m", hint))
    }
}

impl Validator for ShellCompleter {}

impl Helper for ShellCompleter {}
