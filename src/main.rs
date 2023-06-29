pub mod error_type;
/// Generates the marquage graph and the NuSMV code from a Petri network
/// ## Inputs
/// Takes the petri network as a :
///
/// ### JSON file
/// #### Example
/// ```json
///     {
///   "m_names": [
///     "A",
///     "B",
///     "C"
///   ],
///   "m_init": [
///     1,
///     0,
///     2
///   ],
///   "transitions": [
///     [
///       [
///         0,
///         0
///       ],
///       [
///         -2,
///         -2
///       ],
///       [
///         2,
///         0
///       ]
///     ],
///     [
///       [
///         -1,
///         -1
///       ],
///       [
///         1,
///         0
///       ],
///       [
///         0,
///         0
///       ]
///     ]
///   ]
/// }
/// ```
/// ### As a Tina Ndr file:
/// #### Example
/// p 215.0 210.0 p0 4 n
/// p 30.0 50.0 p1 1 n
/// t 55.0 180.0 t0 0 w n
/// t 185.0 60.0 t1 0 w n
/// e t1 p1 1 n
/// e p0 t1 1 n
/// e t0 p1 1 n
/// e p0 t0 2 n
/// h test
pub mod graph_gen;
/// Module used to parse ndr file
pub mod ndr_parser;
mod petri_parser;

use crate::graph_gen::*;
use clap::*;
use error_type::ErrorTypes;
use graphviz_rust::{cmd::Format, exec, parse, printer::PrinterContext};
use petri_parser::parser::*;
use std::{collections::HashMap, fs};

const DOT_TEMPLATE: &str = r#"
    digraph {
        NAMING
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

#[derive(Debug, Parser)]
/// Program that allows to convert a petri network to a Finite state automata
/// using json to represent petri network and smv to represent the automata
#[command(author, version)]
pub struct Args {
    /// path to the source of the petri network
    #[arg(short,long,default_value_t=String::from("./net.petri"))]
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
        m_names,
        m_init,
        transitions,
    } = if petri.starts_with("{") {
        serde_json::from_str(&petri)?
    } else {
        PetriNet::new(&petri)?.generate_input()
    };

    if transitions.iter().any(|t| t.len() != m_init.len()) {
        return Err(anyhow::Error::new(ErrorTypes::TransitionSizeNotMatching {
            expected: m_init.len(),
        }));
    }

    // CONSTRUCTION DU GRAPH DES MARQUAGES
    let mut marquage_graph = HashMap::new();
    generate_graph(m_init.clone(), &transitions, &mut marquage_graph);

    // GENERATION DES BORNES DES PLACES
    let places = m_init
        .iter()
        .zip(&m_names)
        .enumerate()
        .map(|(i, (v, s))| Place {
            alias: s.clone(),
            indice: i,
            max: v.unwrap(),
            min: v.unwrap(),
        })
        .collect::<Vec<_>>();

    let places = marquage_graph.keys().fold(places, |ps, k| {
        ps.iter().zip(k).map(|(p, x)| p.update(*x)).collect()
    });

    let code_template = CODE_TEMPLATE
        .replace(
            "STATES",
            &format!(
                "{{{}}}",
                marquage_graph
                    .keys()
                    .map(|m| format!("s_{}", vector_to_string(m, "_")))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        )
        .replace(
            "PLACES",
            &places
                .iter()
                .map(|p| {
                    format!(
                        "\t\t{} : {};",
                        p.alias,
                        /*p.min,*/
                        if p.max == p.min {
                            format!("0..{}", p.max)
                        } else {
                            format!("{}..{}", p.min, p.max)
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .replace(
            "STATE_ASSIGN",
            &format!("s_{}", vector_to_string(&m_init, "_")),
        )
        .replace(
            "STATE_TRANSITION",
            &marquage_graph
                .iter()
                .map(|(current, next)| {
                    format!(
                        "\t\ts={} : {{{}}};",
                        format!("s_{}", vector_to_string(current, "_")),
                        if next.len() > 0 {
                            next.iter()
                                .map(|(_, v)| format!("s_{}", vector_to_string(v, "_")))
                                .collect::<Vec<_>>()
                                .join(",")
                        } else {
                            "s".to_string()
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .replace(
            "PLACE_TRANSITION",
            &places
                .iter()
                .map(|p| {
                    format!(
                        "{} := case\n {} \n\t\tesac;",
                        p.alias,
                        marquage_graph
                            .keys()
                            .map(|current| format!(
                                "\t\ts=s_{} : {{{}}};",
                                vector_to_string(current, "_"),
                                match current[p.indice] {
                                    Some(x) => x,
                                    None => p.max,
                                }
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
        let dot_template = DOT_TEMPLATE
            .replace("NAMING", &format!("\"{}\"", m_names.join("-")))
            .replace(
                "GRAPH",
                &marquage_graph
                    .iter()
                    .map(|(k, v)| {
                        v.iter()
                            .map(|(t, n)| {
                                format!(
                                    " \"{}\" -> \"{}\" [label = \"t{}\"]",
                                    vector_to_string(k, "-"),
                                    vector_to_string(n, "-"),
                                    transitions
                                        .iter()
                                        .enumerate()
                                        .find(|(_, x)| x.iter().zip(t).all(|((x, _), t)| t == x))
                                        .unwrap()
                                        .0
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

        // fs::write(format!("{}.dot", &p), dot_template)?;
        fs::write(format!("{}.svg", &p), graph_svg)?;
        open::that(&format!("{}.svg", &p))?;
    };

    Ok(())
}
