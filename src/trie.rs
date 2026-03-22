use std::collections::HashMap;

#[derive(Default)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end_of_word: bool,
    word: Option<String>,
}

pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::default(),
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.is_end_of_word = true;
        node.word = Some(word.to_string());
    }

    pub fn find_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut node = &self.root;

        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(child) => node = child,
                None => return vec![],
            }
        }

        let mut results = Vec::new();
        self.collect_words(node, &mut results);
        results
    }

    fn collect_words(&self, node: &TrieNode, results: &mut Vec<String>) {
        if let Some(ref word) = node.word {
            results.push(word.clone());
        }

        for child in node.children.values() {
            self.collect_words(child, results);
        }
    }
}
