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
        msg.split_ascii_whitespace()
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

    fn extract_final_token(msg: &str) -> Option<Token> {
        msg.split_ascii_whitespace()
            .last()
            .map(Self::normalize_token)
    }

    fn random_token(&self, rand: &mut fastrand::Rng) -> Option<Token> {
        let len = self.0.len();
        if len == 0 {
            None
        } else {
            let i = rand.usize(..len);
            self.0.keys().nth(i).cloned()
        }
    }

    pub fn ingest(&mut self, msg: &str) -> bool {
        let mut learned_new_word = false;
        // This is a silly way to do windows rust ppl :sob:
        let _ = Self::parse(msg)
            .map_windows(|[from, to]| {
                if let Some(edge) = self.0.get_mut(from) {
                    edge.increment_token(to);
                } else {
                    let new = Edges(HashMap::from_iter([(to.clone(), 1)]), 1);
                    self.0.insert(from.clone(), new);
                    learned_new_word = true;
                }
            })
            .collect::<Vec<_>>();

        learned_new_word
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

    pub fn respond(
        &self,
        msg: &str,
        is_self: bool,
        mut typing_oneshot: Option<TypingSender>,
    ) -> Option<String> {
        const MAX_TOKENS: usize = 20;

        let mut rng = fastrand::Rng::new();

        // Roll if we should reply
        if !Self::should_reply(&mut rng, is_self) {
            debug!("Failed roll");
            return None;
        }

        // Get our final token, or a random one if the message has nothing, or don't reply at all
        // if we have no tokens at all.
        let last_token = Self::extract_final_token(msg).or_else(|| self.random_token(&mut rng))?;
        let mut current_token = &last_token;

        let mut chain = Vec::with_capacity(MAX_TOKENS);
        let mut has_triggered_typing = false;

        while current_token.is_some() && chain.len() <= MAX_TOKENS {
            if let Some(edges) = self.0.get(current_token) {
                let next = edges.sample(&mut rng, chain.len() > 2);

                if let Some(ref tok) = next {
                    if let Some(s) = tok {
                        // Is this a non-ending token? If so, push it to our chain!
                        chain.push(s.clone());
                        if !has_triggered_typing && let Some(typ) = typing_oneshot.take() {
                            typ.send(true).ok();
                        }
                        current_token = tok;
                    } else {
                        // If we reached an end token, stop chaining
                        break;
                    }
                } else {
                    // If we failed to sample any tokens, we can't continue the chain
                    break;
                }
            } else {
                // If we don't know the current word, we can't continue the chain
                break;
            }
        }

        if let Some(typ) = typing_oneshot.take() {
            typ.send(false).ok();
        }

        Some(chain.join(" "))
    }

    pub fn word_count(&self) -> usize {
        self.0.len()
    }

    pub fn get_weights(&self, tok: &str) -> Option<&Edges> {
        self.0.get(&Self::normalize_token(tok))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::default::Default;

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
        let reply = brain.respond("hello", false, None);
        assert_eq!(reply, Some("world".to_string()));
    }

    #[test]
    fn at_least_2_tokens() {
        let mut brain = Brain::default();
        brain.ingest("hello world");
        brain.ingest("hello");
        brain.ingest("hello");
        brain.ingest("hello");

        for _ in 0..100 {
            // I'm too lazy to mock lazyrand LOL!!
            let reply = brain.respond("hello", false, None);
            assert_eq!(reply, Some("world".to_string()));
        }
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
        let reply = brain.respond("a", false, None);
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
}
