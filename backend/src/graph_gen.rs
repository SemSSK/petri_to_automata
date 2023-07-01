use serde::Deserialize;

/// description of the input shape
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Input {
    pub m_names: Vec<String>,
    pub m_init: Vec<Option<i32>>,
    pub transitions: Vec<Vec<(i32, i32)>>,
}

use iter_tools::Itertools;
use std::collections::HashMap;

use crate::output_generators::Output;

/// internal representation of a place node in a petri network
#[derive(Debug, Clone)]
pub struct Place {
    pub alias: String,
    pub indice: usize,
    pub min: i32,
    pub max: i32,
}

impl Place {
    pub fn build(
        m_names: &[String],
        m_init: &[Option<i32>],
        marking_graph: &HashMap<Vec<Option<i32>>, Vec<(Vec<i32>, Vec<Option<i32>>)>>,
    ) -> Vec<Self> {
        let places = m_init
            .iter()
            .zip(m_names)
            .enumerate()
            .map(|(i, (v, s))| Place {
                alias: s.clone(),
                indice: i,
                max: v.unwrap(),
                min: v.unwrap(),
            })
            .collect::<Vec<_>>();

        marking_graph.keys().fold(places, |ps, k| {
            ps.iter().zip(k).map(|(p, x)| p.update(*x)).collect()
        })
    }
    /// updates the k border values (min and max) of the node
    pub fn update(&self, v: Option<i32>) -> Self {
        let Some(v) = v else {
            return Self {
                max: 1000,
                indice: self.indice,
                min: self.min,
                alias: self.alias.to_string(),
            }
        };
        if v > self.max {
            Self {
                max: v,
                indice: self.indice,
                min: self.min,
                alias: self.alias.to_string(),
            }
        } else if v < self.min {
            Self {
                max: self.max,
                indice: self.indice,
                min: v,
                alias: self.alias.to_string(),
            }
        } else {
            self.clone()
        }
    }
}

/// recursive function that get's the parents of a certain node
/// ## Inputs
///
/// ` m: &'a Vec<Option<i32>>` :the marquage to fetch the parents of
///
///  `marquage_graph: &'a HashMap<Vec<Option<i32>>, Vec<(Vec<i32>, Vec<Option<i32>>)>>` :current state of the marquage_graph
///
/// `acc: &mut Vec<&'a Vec<Option<i32>>>` : accumulates the parents and return it
///
/// ## Returns
///
/// a `Vec<&'a Vec<Option<i32>>>` the parents of the node
fn get_parents_of_marking<'a>(
    m: &'a Vec<Option<i32>>,
    marquage_graph: &'a HashMap<Vec<Option<i32>>, Vec<(Vec<i32>, Vec<Option<i32>>)>>,
    acc: &mut Vec<&'a Vec<Option<i32>>>,
) -> Vec<&'a Vec<Option<i32>>> {
    let mut parents: Vec<&Vec<Option<i32>>> = vec![];
    acc.push(&m);
    let keys = marquage_graph
        .iter()
        .filter(|(_, value)| value.iter().map(|(_, v)| v).contains(&m))
        .map(|(key, _)| key)
        .filter(|key| !acc.contains(key))
        .collect::<Vec<_>>();
    parents.append(&mut keys.clone());
    for k in keys {
        let mut local_parents = get_parents_of_marking(k, marquage_graph, acc);
        parents.append(&mut local_parents);
    }
    parents
}

/// main function
///
/// ## Usage
/// Generates the graph recursively by mutating a `HashMap`
/// ## Inputs
/// `m: Vec<Option<i32>>` : the initial marquage
///
/// `transitions: &Vec<Vec<(i32, i32)>>` : the vector of transitions
///
/// ## Returns
/// `()`: use mutation of the `marquage_graph` to store its value   
fn generate_graph(
    m: Vec<Option<i32>>,
    transitions: &Vec<Vec<(i32, i32)>>,
    marquage_graph: &mut HashMap<Vec<Option<i32>>, Vec<(Vec<i32>, Vec<Option<i32>>)>>,
) {
    let mut next_ms = transitions
        .iter()
        .map(|t| {
            (
                t.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
                activate_transition(t, &m),
            )
        })
        .filter(|(_, m)| {
            m.iter().all(|x| match x {
                Some(x) => *x >= 0,
                None => true,
            })
        })
        .collect::<Vec<_>>();

    marquage_graph.insert(m.clone(), next_ms.clone());

    let binding = next_ms.clone();
    let parents = binding
        .iter()
        .map(|(t, n)| {
            (
                (t, n),
                get_parents_of_marking(n, marquage_graph, &mut vec![]),
            )
        })
        .collect::<Vec<_>>();

    next_ms = parents
        .into_iter()
        .map(|((t, n), ps)| {
            ps.into_iter()
                .map(|p| {
                    if p.iter()
                        .zip(n)
                        .all(|(xp, xn)| xn.map_or(true, |xn| xp.map_or(true, |xp| xn - xp >= 0)))
                    {
                        (
                            t.to_vec(),
                            n.iter()
                                .zip(p)
                                .map(|(xn, xp)| match (xn, xp) {
                                    (Some(xn), Some(xp)) => {
                                        if xn - xp > 0 {
                                            None
                                        } else {
                                            Some(*xn)
                                        }
                                    }
                                    (xn, _) => *xn,
                                })
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        (t.to_vec(), n.to_vec())
                    }
                })
                .collect::<Vec<_>>()
        })
        .map(|ns| {
            ns.into_iter().reduce(|(t, a), (_, n)| {
                (
                    t,
                    a.into_iter()
                        .zip(n)
                        .map(|(xa, xn)| match (xa, xn) {
                            (None, _) | (_, None) => None,
                            (_, xn) => xn,
                        })
                        .collect_vec(),
                )
            })
        })
        .flatten()
        .unique()
        .collect::<Vec<_>>();

    marquage_graph.insert(m, next_ms.clone());

    for (_, mi) in next_ms {
        match marquage_graph.get(&mi) {
            Some(_) => (),
            None => generate_graph(mi, transitions, marquage_graph),
        }
    }
}

/// Does the addition between a transition and a marquage
fn activate_transition(transition: &[(i32, i32)], marking: &[Option<i32>]) -> Vec<Option<i32>> {
    transition
        .iter()
        .zip(marking)
        .map(|((x1, x2), y)| y.map(|y| if y - x1 >= 0 { y + x2 - x1 } else { -1 }))
        .collect()
}

pub fn compile_to_output(input: Input) -> Result<Output, anyhow::Error> {
    let Input {
        m_names,
        m_init,
        transitions,
    } = input;

    // CONSTRUCTION DU GRAPH DES MARQUAGES
    let mut marking_graph = HashMap::new();
    generate_graph(m_init.clone(), &transitions, &mut marking_graph);

    // GENERATION DES BORNES DES PLACES
    let places = Place::build(&m_names, &m_init, &marking_graph);

    Output::generate(&m_names, &m_init, &marking_graph, &places, &transitions)
}
