#![allow(unused)]

use std::collections::HashMap;

use log::debug;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

/// Some = Word, None = End Message
pub type Token = Option<String>;
pub type Weight = u16;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Edges(HashMap<Token, Weight>, u64);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Brain(HashMap<Token, Edges>);

pub type TypingSender = oneshot::Sender<bool>;

pub fn format_token(tok: &Token) -> String {
    if let Some(w) = tok {
        w.clone()
    } else {
        "~END".to_string()
    }
}

impl Edges {
    fn increment_token(&mut self, tok: &Token) {
        if let Some(w) = self.0.get_mut(tok) {
            *w = w.saturating_add(1);
        } else {
            self.0.insert(tok.clone(), 1);
        }
        self.1 = self.1.saturating_add(1);
    }

    fn merge_from(&mut self, other: Self) {
        self.0.reserve(other.0.len());
        for (k, v) in other.0.into_iter() {
            if let Some(w) = self.0.get_mut(&k) {
                *w = w.saturating_add(v);
            } else {
                self.0.insert(k, v);
            }
            self.1 = self.1.saturating_add(v as u64);
        }
    }

    fn sample(&self, rand: &mut fastrand::Rng, allow_end: bool) -> Option<&Token> {
        let total_dist = if !allow_end && let Some(weight) = self.0.get(&None) {
            self.1 - *weight as u64
        } else {
            self.1
        };
        let mut dist_left = rand.f64() * total_dist as f64;

        for (tok, weight) in self.0.iter().filter(|(tok, _)| allow_end || tok.is_some()) {
            dist_left -= *weight as f64;
            if dist_left < 0.0 {
                return Some(tok);
            }
        }
        None
    }

    pub fn forget(&mut self, token: &Token) {
        if let Some(w) = self.0.remove(token) {
            self.1 -= w as u64;
        }
    }

    pub fn iter_weights(&self) -> impl Iterator<Item = (&Token, Weight, f64)> {
        self.0
            .iter()
            .map(|(k, v)| (k, *v, (*v as f64) / (self.1 as f64)))
    }
}

const FORCE_REPLIES: bool = cfg!(test) || (option_env!("BINGUS_FORCE_REPLY").is_some());

impl Brain {
    fn normalize_token(word: &str) -> Token {
        let w = if word.starts_with("http://") || word.starts_with("https://") {
            word.to_string()
        } else {
            word.to_ascii_lowercase()
        };
        Some(w)
    }

    fn parse(msg: &str) -> impl Iterator<Item = Token> {
        msg.split_whitespace()
            .filter_map(|w| {
                // Filter out pings, they can get annoying
                if w.starts_with("<@") && w.ends_with(">") {
                    None
                } else {
                    Some(Self::normalize_token(w))
                }
            })
            .chain(std::iter::once(None))
    }

    fn should_reply(rand: &mut fastrand::Rng, is_self: bool) -> bool {
        let chance = if is_self { 45 } else { 80 };
        let roll = rand.u8(0..=100);

        (FORCE_REPLIES) || roll <= chance
    }

    fn extract_final_word(msg: &str) -> Option<String> {
        msg.split_whitespace()
            .last()
            .and_then(Self::normalize_token)
    }

    fn random_token(&self, rand: &mut fastrand::Rng) -> Option<&Token> {
        let len = self.0.len();
        if len == 0 {
            None
        } else {
            let i = rand.usize(..len);
            self.0.keys().nth(i)
        }
    }

    pub fn ingest(&mut self, msg: &str) -> bool {
        // Using reduce instead of .any here to prevent short circuting
        Self::parse(msg)
            .map_windows(|[from, to]| {
                if let Some(edge) = self.0.get_mut(from) {
                    edge.increment_token(to);
                    false
                } else {
                    let new = Edges(HashMap::from_iter([(to.clone(), 1)]), 1);
                    self.0.insert(from.clone(), new);
                    true
                }
            })
            .reduce(|acc, c| acc || c)
            .unwrap_or_default()
    }

    pub fn forget(&mut self, word: &str) {
        let tok = Self::normalize_token(word);

        self.0.remove(&tok);

        for edge in self.0.values_mut() {
            edge.forget(&tok);
        }
    }

    pub fn merge_from(&mut self, other: Self) {
        for (k, v) in other.0.into_iter() {
            if let Some(edges) = self.0.get_mut(&k) {
                edges.merge_from(v);
            } else {
                self.0.insert(k, v);
            }
        }
    }

    fn next_from(&self, tok: &Token, rand: &mut fastrand::Rng, allow_end: bool) -> Option<&Token> {
        // Get the edges for the current token
        // If we have that token, sample its edges
        // Otherwise, if we don't know that token, and allow_end is false, try to pick a random token instead
        self.0
            .get(tok)
            .and_then(|edges| edges.sample(rand, allow_end))
            .or_else(|| {
                if allow_end {
                    None
                } else {
                    self.random_token(rand)
                }
            })
    }

    pub fn respond(
        &self,
        msg: &str,
        is_self: bool,
        force_reply: bool,
        mut typing_oneshot: Option<TypingSender>,
    ) -> Option<String> {
        const MAX_TOKENS: usize = 20;

        let mut rng = fastrand::Rng::new();

        // Roll if we should reply
        if !force_reply && !Self::should_reply(&mut rng, is_self) {
            debug!("Failed roll");
            return None;
        }

        // Get the final token
        let last_token = Self::extract_final_word(msg);

        let mut current_token = if let Some(t) = last_token {
            // We found a word at the end of the previous message
            &Some(t)
        } else {
            // We couldn't find a word at the end of the last message, pick a random one
            // If we *still* don't have a token, return early
            self.random_token(&mut rng)?
        };

        let mut chain = Vec::with_capacity(MAX_TOKENS);
        let sep = String::from(" ");

        while let Some(next @ Some(s)) = self.next_from(current_token, &mut rng, !chain.is_empty())
            && chain.len() <= MAX_TOKENS
        {
            chain.push(s);
            if let Some(typ) = typing_oneshot.take() {
                typ.send(true).ok();
            }
            current_token = next;
        }

        if let Some(typ) = typing_oneshot.take() {
            typ.send(false).ok();
        }

        if chain.is_empty() {
            None
        } else {
            let s = chain
                .into_iter()
                .intersperse(&sep)
                .cloned()
                .collect::<String>();
            Some(s)
                .filter(|s| !s.trim().is_empty())
                .filter(|s| s.encode_utf16().count() < 2000)
        }
    }

    pub fn word_count(&self) -> usize {
        self.0.len()
    }

    pub fn get_weights(&self, tok: &str) -> Option<&Edges> {
        self.0
            .get(&Self::normalize_token(tok))
            .filter(|e| !e.0.is_empty())
    }

    fn legacy_token_format(tok: &Token) -> String {
        tok.as_ref()
            .map(|s| format!("W-{s}"))
            .unwrap_or_else(|| String::from("E--"))
    }

    pub fn as_legacy_hashmap(&self) -> HashMap<String, HashMap<String, Weight>> {
        self.0
            .iter()
            .map(|(k, v)| {
                let map =
                    v.0.iter()
                        .map(|(t, w)| (Self::legacy_token_format(t), *w))
                        .collect();
                (Self::legacy_token_format(k), map)
            })
            .collect()
    }

    fn read_legacy_token(s: String) -> Token {
        match s.as_str() {
            "E--" => None,
            word => Some(word.strip_prefix("W-").unwrap_or(word).to_string()),
        }
    }

    pub fn from_legacy_hashmap(map: HashMap<String, HashMap<String, Weight>>) -> Self {
        Self(
            map.into_iter()
                .map(|(k, v)| {
                    let sum = v.values().map(|w| *w as u64).sum::<u64>();
                    let edges = Edges(
                        v.into_iter()
                            .map(|(t, w)| (Self::read_legacy_token(t), w))
                            .collect(),
                        sum,
                    );
                    (Self::read_legacy_token(k), edges)
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::default::Default;

    extern crate test;

    use test::Bencher;

    #[test]
    fn ingest_parse() {
        let tokens = Brain::parse("Hello world").collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![Some("hello".to_string()), Some("world".to_string()), None]
        );
    }

    #[test]
    fn ingest_url() {
        let tokens = Brain::parse("https://example.com/CAPS-PATH").collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![Some("https://example.com/CAPS-PATH".to_string()), None]
        );
    }

    #[test]
    fn ingest_ping() {
        let tokens = Brain::parse("hi <@1234567>").collect::<Vec<_>>();
        assert_eq!(tokens, vec![Some("hi".to_string()), None]);
    }

    #[test]
    fn basic_chain() {
        let mut brain = Brain::default();
        brain.ingest("hello world");
        let hello_edges = brain
            .0
            .get(&Some("hello".to_string()))
            .expect("Hello edges not created");
        assert_eq!(
            hello_edges.0,
            HashMap::from_iter([(Some("world".to_string()), 1)])
        );
        let reply = brain.respond("hello", false, false, None);
        assert_eq!(reply, Some("world".to_string()));
    }

    #[test]
    fn at_least_1_token() {
        let mut brain = Brain::default();
        brain.ingest("hello world");
        for _ in 0..100 {
            brain.ingest("hello");
        }

        for _ in 0..100 {
            // I'm too lazy to mock lazyrand LOL!!
            let reply = brain.respond("hello", false, false, None);
            assert_eq!(reply, Some("world".to_string()));
        }
    }

    #[test]
    fn forget_word() {
        let mut brain = Brain::default();

        brain.ingest("hello world");
        brain.ingest("hello evil world");

        brain.forget("evil");

        assert!(
            !brain.0.contains_key(&Some(String::from("evil"))),
            "Edges still exist for evil"
        );
        let edges = brain
            .0
            .get(&Some(String::from("hello")))
            .expect("No weights for hello");
        assert!(
            !edges.0.contains_key(&Some(String::from("evil"))),
            "Edges for hello still has evil"
        );
        assert_eq!(edges.1, 1);
    }

    #[test]
    fn none_on_empty() {
        let mut brain = Brain::default();

        let reply = brain.respond("hello", false, false, None);
        assert_eq!(reply, None);
    }

    #[test]
    fn none_on_long() {
        let mut brain = Brain::default();

        let msg = vec!["a"; 2500].into_iter().collect::<String>();
        let msg = format!("hello {msg}");

        brain.ingest(&msg);

        assert!(brain.respond("hello", false, false, None).is_none())
    }

    #[test]
    fn random_on_end() {
        let mut brain = Brain::default();
        brain.ingest("world hello");

        let reply = brain.respond("hello", false, false, None);
        assert!(reply.is_some());
    }

    #[test]
    fn long_chain() {
        const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
        let msg = LETTERS
            .chars()
            .map(|c| c.to_string())
            .intersperse(" ".to_string())
            .collect::<String>();
        let mut brain = Brain::default();
        brain.ingest(&msg);
        let reply = brain.respond("a", false, false, None);
        let expected = LETTERS
            .chars()
            .skip(1)
            .take(21)
            .map(|c| c.to_string())
            .intersperse(" ".to_string())
            .collect::<String>();
        assert_eq!(reply, Some(expected));
    }

    #[test]
    fn merge_brain() {
        let mut brain1 = Brain::default();
        let mut brain2 = Brain::default();

        brain1.ingest("hello world");
        brain2.ingest("hello world");
        brain2.ingest("hello world");
        brain2.ingest("other word");

        brain1.merge_from(brain2);

        let hello_edges = brain1
            .0
            .get(&Some("hello".to_string()))
            .expect("Hello edges not created");
        assert_eq!(
            hello_edges.0,
            HashMap::from_iter([(Some("world".to_string()), 3)])
        );

        let new_edges = brain1
            .0
            .get(&Some("other".to_string()))
            .expect("New edges not created");
        assert_eq!(
            new_edges.0,
            HashMap::from_iter([(Some("word".to_string()), 1)])
        );
    }

    #[bench]
    fn bench_learn(b: &mut Bencher) {
        b.iter(|| {
            let mut brain = Brain::default();
            brain.ingest(
                "your name is bingus the discord bot and this message is a test for benchmarking",
            );
        });
    }

    #[bench]
    fn bench_respond(b: &mut Bencher) {
        let mut brain = Brain::default();
        brain.ingest(
            "your name is bingus the discord bot and this message is a test for benchmarking",
        );
        b.iter(|| {
            brain.respond("your", false, true, None);
        });
    }

    include!("lorem.rs");

    #[bench]
    fn bench_learn_large(b: &mut Bencher) {
        b.iter(|| {
            let mut brain = Brain::default();
            brain.ingest(LOREM);
        });
    }
}
