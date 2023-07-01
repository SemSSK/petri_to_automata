use anyhow::Error;
use iter_tools::Itertools;
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;

use crate::{error_type::ErrorTypes, graph_gen::Input};

#[derive(Debug, Parser)]
#[grammar = "parser.pest"]
pub struct PestParser;

type Identifier = String;

#[derive(Debug)]
struct Place {
    name: Identifier,
    tokens: i32,
}

impl Place {
    pub fn from_rule(inner_rules: &mut Pairs<'_, Rule>) -> Self {
        Place {
            name: inner_rules.next().unwrap().as_str().to_string(),
            tokens: inner_rules.next().unwrap().as_str().parse::<i32>().unwrap(),
        }
    }
}

trait NoPlaceRepetition {
    fn check_for_place_repetion(&self) -> Result<(), ErrorTypes>;
}

impl NoPlaceRepetition for Vec<Place> {
    fn check_for_place_repetion(&self) -> Result<(), ErrorTypes> {
        if self
            .iter()
            .map(|p| &p.name)
            .unique()
            .collect::<Vec<_>>()
            .len()
            == self.len()
        {
            Ok(())
        } else {
            Err(ErrorTypes::BadPlace)
        }
    }
}

#[derive(Debug)]
struct Entry(String, i32);

impl Entry {
    pub fn from_rule(inner_rules: &mut Pairs<'_, Rule>) -> Self {
        Entry(
            inner_rules.next().unwrap().as_str().to_string(),
            match inner_rules.next() {
                Some(i) => i.as_str().parse::<i32>().unwrap(),
                None => 1,
            },
        )
    }
    pub fn place_exists(&self, place: &[Place], t_id: &str) -> Result<(), ErrorTypes> {
        match place.iter().any(|p| p.name == self.0) {
            true => Ok(()),
            false => Err(ErrorTypes::BadTransition {
                reason: format!(
                    "Reference to Undeclared place {} in transition {}",
                    self.0, t_id
                ),
            }),
        }
    }
}

#[derive(Debug)]
struct Transition {
    name: Identifier,
    inputs: Vec<Entry>,
    outputs: Vec<Entry>,
}

fn non_repeating_entry(entries: &[Entry]) -> Result<(), ErrorTypes> {
    if entries
        .iter()
        .map(|e| &e.0)
        .unique()
        .collect::<Vec<_>>()
        .len()
        == entries.len()
    {
        Ok(())
    } else {
        Err(ErrorTypes::BadTransition {
            reason: "Repeating places".to_string(),
        })
    }
}

impl Transition {
    pub fn from_rule(inner_rules: &mut Pairs<'_, Rule>) -> Self {
        Transition {
            name: inner_rules.next().unwrap().as_str().to_string(),
            inputs: inner_rules
                .next()
                .unwrap()
                .into_inner()
                .into_iter()
                .map(|r| Entry::from_rule(&mut r.into_inner()))
                .collect(),
            outputs: inner_rules
                .next()
                .unwrap()
                .into_inner()
                .into_iter()
                .map(|r| Entry::from_rule(&mut r.into_inner()))
                .collect(),
        }
    }
    pub fn validate_transition(&self, places: &[Place]) -> Result<(), ErrorTypes> {
        non_repeating_entry(&self.inputs)?;
        non_repeating_entry(&self.outputs)?;
        for entry in &self.inputs {
            entry.place_exists(places, &self.name)?;
        }
        for entry in &self.outputs {
            entry.place_exists(places, &self.name)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct PetriNet {
    places: Vec<Place>,
    transitions: Vec<Transition>,
}

impl PetriNet {
    pub fn new(code: &str) -> Result<Self, Error> {
        let rules = PestParser::parse(Rule::petri_net, code)?.next().unwrap();
        let net = Self::from_rule(&mut rules.into_inner());
        net.validate_petri_net()?;
        Ok(net)
    }
    fn from_rule(inner_rule: &mut Pairs<'_, Rule>) -> Self {
        let mut transitions = vec![];
        let mut places = vec![];
        for line in inner_rule {
            match line.as_rule() {
                Rule::place => places.push(Place::from_rule(&mut line.into_inner())),
                Rule::transition => transitions.push(Transition::from_rule(&mut line.into_inner())),
                Rule::EOI => (),
                _ => unreachable!(),
            }
        }
        PetriNet {
            places,
            transitions,
        }
    }
    fn validate_petri_net(&self) -> Result<(), Error> {
        self.places.check_for_place_repetion()?;
        for t in &self.transitions {
            t.validate_transition(&self.places)?;
        }
        Ok(())
    }
    pub fn generate_input(self) -> Input {
        let m_init = self
            .places
            .iter()
            .map(|place| Some(place.tokens))
            .collect::<Vec<_>>();

        let transitions = self
            .transitions
            .into_iter()
            .map(|transition| {
                self.places
                    .iter()
                    .map(|p| {
                        let input = match transition
                            .inputs
                            .iter()
                            .find(|&Entry(name, _)| name == &p.name)
                        {
                            Some(&Entry(_, i)) => i,
                            None => 0,
                        };
                        let output = match transition
                            .outputs
                            .iter()
                            .find(|&Entry(name, _)| name == &p.name)
                        {
                            Some(&Entry(_, o)) => o,
                            None => 0,
                        };
                        (input, output)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let m_names = self
            .places
            .into_iter()
            .map(|place| place.name)
            .collect::<Vec<_>>();

        Input {
            m_names,
            m_init,
            transitions,
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::PetriNet;

    #[test]
    fn test_reading() {
        let file = fs::read_to_string("./net.petri").unwrap();
        let net = PetriNet::new(&file).unwrap();
        println!("{:?}", net);
        assert!(true)
    }
}
