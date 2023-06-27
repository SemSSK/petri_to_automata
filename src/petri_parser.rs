use anyhow::Error;
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;

use crate::graph_gen::{ErrorTypes, Input};

#[derive(Debug, Parser)]
#[grammar = "parser.pest"]
pub struct PestParser;

#[derive(Debug, Clone)]
struct Transition {
    name: String,
    inputs: Vec<(String, i32)>,
    outputs: Vec<(String, i32)>,
}

#[derive(Debug)]
struct Place {
    name: String,
    tokens: i32,
}

/// Constructs Abstract syntax tree from Raw file
fn read_file(file_content: &str) -> Result<(Vec<Place>, Vec<Transition>), Error> {
    let file = PestParser::parse(Rule::petri_net, &file_content)?
        .next()
        .unwrap();

    let mut transitions = vec![];
    let mut places = vec![];

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::place => {
                places.push(extract_place(&mut line.into_inner()));
            }
            Rule::transition => {
                transitions.push(extract_transition(&mut line.into_inner()));
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    Ok((places, transitions))
}

/// Abstration over place extraction to reduce boilder-plate
fn extract_place(inner_rules: &mut Pairs<'_, Rule>) -> Place {
    Place {
        name: inner_rules.next().unwrap().as_str().to_string(),
        tokens: inner_rules.next().unwrap().as_str().parse::<i32>().unwrap(),
    }
}

/// Converts transition Rule to Transition struct
fn extract_transition(inner_rules: &mut Pairs<'_, Rule>) -> Transition {
    Transition {
        name: inner_rules.next().unwrap().as_str().to_string(),
        inputs: extract_entries(inner_rules),
        outputs: extract_entries(inner_rules),
    }
}

/// Abstration over entries extraction to reduce boilder plate
fn extract_entries(inner_rules: &mut Pairs<'_, Rule>) -> Vec<(String, i32)> {
    inner_rules
        .next()
        .unwrap()
        .into_inner()
        .into_iter()
        .map(|entry| {
            let mut rule = entry.into_inner();
            (
                rule.next().unwrap().as_str().to_string(),
                match rule.next() {
                    Some(v) => v.as_str().parse::<i32>().unwrap(),
                    None => 1,
                },
            )
        })
        .collect()
}

/// Validates transition places
fn validate_transitions(
    (places, transitions): (Vec<Place>, Vec<Transition>),
) -> Result<(Vec<Place>, Vec<Transition>), ErrorTypes> {
    match transitions
        .iter()
        .map(|t| {
            t.inputs
                .iter()
                .all(|(p_name, _)| places.iter().find(|p| p.name == *p_name).is_some())
                && t.inputs
                    .iter()
                    .all(|(p_name, _)| places.iter().find(|p| p.name == *p_name).is_some())
        })
        .all(|p| p)
    {
        true => Ok((places, transitions)),
        false => Err(ErrorTypes::PlaceNotDeclared),
    }
}

/// Generates the inputs read by the backend (transition matrix + initial marking vector)
fn generate_input((places, transitions): (Vec<Place>, Vec<Transition>)) -> Input {
    let m_init = places
        .iter()
        .map(|place| Some(place.tokens))
        .collect::<Vec<_>>();

    let transitions = transitions
        .into_iter()
        .map(|transition| {
            places
                .iter()
                .map(|p| {
                    let input = match transition.inputs.iter().find(|&(name, _)| name == &p.name) {
                        Some(&(_, i)) => i,
                        None => 0,
                    };
                    let output = match transition.outputs.iter().find(|&(name, _)| name == &p.name)
                    {
                        Some(&(_, o)) => o,
                        None => 0,
                    };
                    (input, output)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let m_names = places
        .into_iter()
        .map(|place| place.name)
        .collect::<Vec<_>>();

    Input {
        m_names,
        m_init,
        transitions,
    }
}

/// Exposed frontend of the parser
pub fn parse_petri_file_to_input(file_content: &str) -> Result<Input, Error> {
    let content = read_file(file_content)?;
    let validated_content = validate_transitions(content)?;
    let input = generate_input(validated_content);
    Ok(input)
}

#[cfg(test)]
mod test {

    use std::fs;

    use crate::{graph_gen::Input, petri_parser::parse_petri_file_to_input};

    #[test]
    fn t() {
        let actual =
            parse_petri_file_to_input(&fs::read_to_string("./net.petri").unwrap()).unwrap();
        let expected = Input {
            m_names: vec!["p0".to_string(), "p1".to_string(), "p2".to_string()],
            m_init: vec![Some(1), Some(2), Some(0)],
            transitions: vec![vec![(1, 1), (2, 0), (0, 2)], vec![(0, 0), (0, 1), (1, 0)]],
        };
        println!("{:?}", actual);
        assert_eq!(expected, actual);
    }
}
