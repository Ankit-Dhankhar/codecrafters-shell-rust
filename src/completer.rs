use rustyline::{
    Context, Helper,
    completion::{Completer, Pair},
    error::ReadlineError, hint::Hinter,
    highlight::Highlighter, validate::Validator
};

use crate::builtins::BUILTINS;
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
            let matches: Vec<Pair> = self
                .trie
                .find_with_prefix(prefix)
                .into_iter()
                .filter(|word| word != prefix)
                .map(|word| Pair {
                    display: word.clone(),
                    replacement: format!("{} ", word),
                })
                .collect();

            return Ok((word_start, matches));
        }

        Ok((word_start, vec![]))
    }
}

impl Hinter for ShellCompleter {
    type Hint = String;
}

impl Highlighter for ShellCompleter {}

impl Validator for ShellCompleter {}

impl Helper for ShellCompleter {}
