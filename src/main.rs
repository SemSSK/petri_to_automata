use clap::*;
use graphviz_rust::{cmd::Format, exec, parse, printer::PrinterContext};
use petri_to_automata::*;
use std::{collections::HashMap, fs};

const DOT_TEMPLATE: &str = r#"
    digraph {
        GRAPH
    }
"#;

const CODE_TEMPLATE: &str = r#"
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

#[derive(Debug, Parser)]
/// Program that allows to convert a petri network to a Finite state automata
/// using json to represent petri network and smv to represent the automata
#[command(author, version)]
struct Args {
    /// path to the source of the petri network
    #[arg(short,long,default_value_t=String::from("./petri.json"))]
    source: String,
    /// path to the output file
    #[arg(short,long,default_value_t=String::from("./automata.smv"))]
    output: String,
    /// path to optional output as a dot file readable by graphviz
    #[arg(short, long)]
    dot: Option<String>,
}

fn main() -> Result<(), anyhow::Error> {
    // READING INPUTS
    let args = Args::parse();
    let petri = fs::read_to_string(&args.source)?;
    let Input {
        m_init,
        transitions,
    } = if petri.starts_with("{") {
        serde_json::from_str(&petri)?
    } else {
        get_input_from_ndr(&petri)
    };

    if transitions.iter().any(|t| t.len() != m_init.len()) {
        return Err(anyhow::Error::new(ErrorTypes::TransitionSizeNotMatching {
            expected: m_init.len(),
        }));
    }

    for t in &transitions {
        if t.iter().all(|x| *x >= 0) {
            return Err(anyhow::Error::new(ErrorTypes::PotentialInfiniteGraph {
                transition: t.to_vec(),
            }));
        }
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
            max: *v,
            min: *v,
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

    let code_template = code_template.replace("STATE_ASSIGN", &vector_to_string(&m_init, ""));

    let code_template = code_template.replace(
        "STATE_TRANSITION",
        &marquage_graph
            .iter()
            .map(|(current, next)| {
                format!(
                    "\t\ts={} : {{{}}};",
                    vector_to_string(current, ""),
                    next.iter()
                        .map(|v| vector_to_string(v, ""))
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

    fs::write(&args.output, code_template)?;

    // generating graph using graphviz
    if let Some(p) = args.dot {
        let dot_template = DOT_TEMPLATE.replace(
            "GRAPH",
            &marquage_graph
                .iter()
                .map(|(k, v)| {
                    v.iter()
                        .map(|n| {
                            format!(
                                " \"{}\" -> \"{}\"",
                                vector_to_string(k, "-"),
                                vector_to_string(n, "-")
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(";\n")
                })
                .collect::<Vec<_>>()
                .join("\n\t\t"),
        );

        let graph =
            parse(&dot_template).map_err(|e| ErrorTypes::CannotAssembleGraph { reason: e })?;
        let graph_svg = exec(
            graph,
            &mut PrinterContext::default(),
            vec![Format::Svg.into()],
        )?;

        fs::write(format!("{}.dot", &p), dot_template)?;
        fs::write(format!("{}.svg", &p), graph_svg)?
    };

    Ok(())
}
