use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{self, ErrorKind}, env,
};

// MADE IN 1h 40min

use serde::Deserialize;

const CODE_TEMPLATE: &'static str = r#"
MODULE main
    VAR
        s : STATES;
PLACES
    ASSIGN
        init(s) := STATE_ASSIGN;

        next(s) := case
STATE_TRANSITION
        esac;
        
        PLACE_TRANSITION

"#;

#[derive(Debug, Clone)]
struct Place {
    alias: String,
    indice: usize,
    min: i32,
    max: i32,
}

impl Place {
    fn update(&self, v: i32) -> Self {
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

fn generate_graph(
    m: Vec<i32>,
    transitions: &Vec<Vec<i32>>,
    marquage_graph: &mut HashMap<Vec<i32>, Vec<Vec<i32>>>,
) {
    let next_ms = transitions
        .iter()
        .map(|t| add_vector(t, &m))
        .filter(|m| m.iter().all(|x| *x >= 0))
        .collect::<Vec<Vec<_>>>();

    marquage_graph.insert(m, next_ms.clone());

    for mi in next_ms {
        match marquage_graph.get(&mi) {
            Some(_) => (),
            None => generate_graph(mi, transitions, marquage_graph),
        }
    }
}

fn add_vector(u: &Vec<i32>, v: &Vec<i32>) -> Vec<i32> {
    u.iter().zip(v).map(|(x, y)| x + y).collect()
}

fn vector_to_string(v: &Vec<i32>, sep: &str) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}

#[derive(Deserialize)]
struct Input {
    m_init: Vec<i32>,
    transitions: Vec<Vec<i32>>,
}

fn main() -> Result<(), impl Error> {
    // READING INPUTS
    let mut args = env::args();
    args.next();
    let file_path = args.next().unwrap_or("./petri.json".into());
    let output_path = args.next().unwrap_or("./automata.smv".into());
    let petri = fs::read_to_string(&file_path)?;
    let Input {
        m_init,
        transitions,
    } = serde_json::from_str(&petri).unwrap();

    if transitions.iter().any(|t| t.len() != m_init.len()) {
        return Err(io::Error::new(
            ErrorKind::Other,
            "Transition input wrong size",
        ));
    }

    // CONSTRUCTION DU GRAPH DES MARQUAGES
    let mut marquage_graph = HashMap::new();
    generate_graph(m_init.clone(), &transitions, &mut marquage_graph);
    // println!("{:?}", marquage_graph);

    // GENERATION DES BORNES DES PLACES
    let places = m_init
        .iter()
        .enumerate()
        .map(|(i, v)| Place {
            alias: format!("p{}", i),
            indice: i,
            max: v.clone(),
            min: v.clone(),
        })
        .collect::<Vec<_>>();

    let places = marquage_graph.keys().fold(places, |ps, k| {
        ps.iter().zip(k).map(|(p, x)| p.update(*x)).collect()
    });

    // println!("{:?}", places);

    let code_template = CODE_TEMPLATE.replace(
        "STATES",
        &format!(
            "{{{}}}",
            marquage_graph
                .keys()
                .map(|m| vector_to_string(m, ""))
                .collect::<Vec<_>>()
                .join(",")
        ),
    );

    let code_template = code_template.replace(
        "PLACES",
        &places
            .iter()
            .map(|p| format!("\t\t{} : 0..{};", p.alias, /*p.min,*/ p.max))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    let code_template = code_template.replace(
        "STATE_ASSIGN",
        &format!("{}", vector_to_string(&m_init, "")),
    );

    let code_template = code_template.replace(
        "STATE_TRANSITION",
        &marquage_graph
            .iter()
            .map(|(current, next)| {
                format!(
                    "\t\ts={} : {{{}}};",
                    vector_to_string(&current, ""),
                    next.iter()
                        .map(|v| vector_to_string(&v, ""))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    );

    let code_template = code_template.replace(
        "PLACE_TRANSITION",
        &places
            .iter()
            .map(|p| {
                format!(
                    "next({}) := case\n {} \n\t\tesac;",
                    p.alias,
                    marquage_graph
                        .iter()
                        .map(|(current, next)| format!(
                            "\t\ts={} : {{{}}};",
                            vector_to_string(current, ""),
                            vector_to_string(
                                &next.iter().map(|x| x[p.indice]).collect::<Vec<_>>(),
                                ","
                            )
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\t\t"),
    );

    fs::write(&output_path, &code_template)
}
