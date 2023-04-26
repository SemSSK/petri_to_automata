use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorTypes {
    #[error("Transition vector size not matching number of places: expected {expected:?}")]
    TransitionSizeNotMatching { expected: usize },
    #[error("Cannot assemble graph reason: {reason:?}")]
    CannotAssembleGraph { reason: String },
    #[error("Cannot generate graph as the transition {transition:?} will generate a graph of infinite nodes")]
    PotentialInfiniteGraph { transition: Vec<i32> },
}

#[derive(Debug)]
struct Place {
    name: String,
    tokens: i32,
}

#[derive(Debug)]
struct Transition(String);

#[derive(Debug)]
enum Tokens {
    P(Place),
    T(Transition),
}

#[derive(Debug, Clone)]
struct TransitionDetails {
    transition: String,
    place: String,
    value: i32,
}

impl Tokens {
    fn is_named(&self, s: &str) -> bool {
        match self {
            Tokens::P(Place { name, .. }) => s == name,
            Tokens::T(Transition(name)) => s == name,
        }
    }
}

fn parse_relations(code: &str, declaration: &Vec<Tokens>) -> Vec<TransitionDetails> {
    code.lines()
        .filter(|l| l.starts_with('e'))
        .map(|l| {
            let words = l.split_whitespace().collect::<Vec<_>>();
            match declaration
                .iter()
                .find(|d| d.is_named(words.get(1).unwrap()))
                .unwrap()
            {
                Tokens::P(_) => TransitionDetails {
                    place: words.get(1).unwrap().to_string(),
                    transition: words.get(2).unwrap().to_string(),
                    value: -(words.get(3).unwrap().to_string().parse::<i32>().unwrap()),
                },
                Tokens::T(_) => TransitionDetails {
                    place: words.get(2).unwrap().to_string(),
                    transition: words.get(1).unwrap().to_string(),
                    value: (words.get(3).unwrap().to_string().parse::<i32>().unwrap()),
                },
            }
        })
        .fold(vec![], move |acc, r| {
            match acc
                .iter()
                .find(|ar| ar.place == r.place && ar.transition == r.place)
            {
                Some(ar) => {
                    let mut result = acc
                        .clone()
                        .into_iter()
                        .filter(|r| r.place != ar.place && r.transition != ar.transition)
                        .collect::<Vec<_>>();
                    let new_transition = TransitionDetails {
                        place: ar.place.to_string(),
                        transition: ar.transition.to_string(),
                        value: ar.value + r.value,
                    };
                    result.push(new_transition);
                    result
                }
                None => {
                    let mut result = acc.clone();
                    result.push(r);
                    result
                }
            }
        })
}

fn parse_tokens_declarations(code: &str) -> Vec<Tokens> {
    code.lines()
        .filter(|l| l.starts_with('t') || l.starts_with('p'))
        .map(|l| {
            let words = l.split_whitespace().collect::<Vec<_>>();
            match words.get(0).unwrap() {
                &"t" => Tokens::T(Transition(words.get(3).unwrap().to_string())),
                _ => Tokens::P(Place {
                    name: words.get(3).unwrap().to_string(),
                    tokens: words.get(4).unwrap().to_string().parse().unwrap(),
                }),
            }
        })
        .collect::<Vec<_>>()
}

fn extract_m_init(tokens: &Vec<Tokens>) -> Vec<i32> {
    tokens
        .iter()
        .filter_map(|t| match t {
            Tokens::P(Place { tokens, .. }) => Some(*tokens),
            Tokens::T(_) => None,
        })
        .collect::<Vec<_>>()
}

fn extract_m_names(tokens: &Vec<Tokens>) -> Vec<&String> {
    tokens
        .iter()
        .filter_map(|t| match t {
            Tokens::P(Place { name, .. }) => Some(name),
            Tokens::T(_) => None,
        })
        .collect::<Vec<_>>()
}

fn extract_transitions(
    tokens: &Vec<Tokens>,
    transitions: &Vec<TransitionDetails>,
) -> Vec<Vec<i32>> {
    let m_names = extract_m_names(tokens);
    tokens
        .iter()
        .filter_map(|t| match t {
            Tokens::P(_) => None,
            Tokens::T(Transition(s)) => Some(s),
        })
        .map(|t| {
            transitions
                .iter()
                .filter(|td| td.transition == *t)
                .collect::<Vec<_>>()
        })
        .map(|tds| {
            m_names
                .iter()
                .map(|name| {
                    tds.iter()
                        .find(|td| td.place == **name)
                        .map(|t| t.value)
                        .unwrap_or(0)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

#[derive(Debug, Deserialize)]
pub struct Input {
    pub m_init: Vec<i32>,
    pub transitions: Vec<Vec<i32>>,
}

pub fn get_input_from_ndr(code: &str) -> Input {
    let tokens = parse_tokens_declarations(code);
    let relations = parse_relations(code, &tokens);
    let m_init = extract_m_init(&tokens);
    let transitions = extract_transitions(&tokens, &relations);
    Input {
        m_init,
        transitions,
    }
}
