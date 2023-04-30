use crate::Input;

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
    input: i32
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
                    input: -(words.get(3).unwrap().to_string().parse::<i32>().unwrap())
                },
                Tokens::T(_) => TransitionDetails {
                    place: words.get(2).unwrap().to_string(),
                    transition: words.get(1).unwrap().to_string(),
                    value: (words.get(3).unwrap().to_string().parse::<i32>().unwrap()),
                    input: 0
                },
            }
        })
        .fold(vec![], move |acc, r| {
            match acc
                .iter()
                .find(|ar| ar.place == r.place && ar.transition == r.transition)
            {
                Some(td) => {
                    let mut result = acc
                        .clone()
                        .into_iter()
                        .filter(|r| 
                            {
                                // dbg!(r.place != td.place);
                                // println!("r = {},{} | td = {},{} , result = {},", r.place,r.transition,td.place,td.transition,(r.place != td.place) || (r.transition != td.transition));
                                r.place != td.place || r.transition != td.transition
                            }
                        )
                        .collect::<Vec<_>>();
                    let new_transition = TransitionDetails {
                        place: td.place.to_string(),
                        transition: td.transition.to_string(),
                        value: td.value + r.value,
                        input: td.input + r.input
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
) -> Vec<Vec<(i32,i32)>> {
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
                        .map(|t| (t.value,t.input))
                        .unwrap_or((0,0))
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

pub fn get_input_from_ndr(code: &str) -> Input {
    let tokens = parse_tokens_declarations(code);
    let relations = parse_relations(code, &tokens);
    let m_init = extract_m_init(&tokens).into_iter().map(|x| Some(x)).collect::<Vec<_>>();
    let transitions = extract_transitions(&tokens, &relations);
    let m_names = extract_m_names(&tokens).into_iter().map(|s| s.clone()).collect::<Vec<_>>();
    Input {
        m_names,
        m_init,
        transitions,
    }
}