use std::{
    collections::HashMap,
    fs, io,
    process::{Command, Stdio},
};

use graphviz_rust::{cmd::Format, printer::PrinterContext};

use crate::{error_type::ErrorTypes, graph_gen::Place};

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

#[derive(Debug)]
pub struct Output {
    smv: String,
    svg: String,
    pub png: Vec<u8>,
}

impl Output {
    pub fn generate(
        m_names: &Vec<String>,
        m_init: &Vec<Option<i32>>,
        marking_graph: &HashMap<Vec<Option<i32>>, Vec<(Vec<i32>, Vec<Option<i32>>)>>,
        places: &Vec<Place>,
        transitions: &Vec<Vec<(i32, i32)>>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            smv: generate_smv_code(m_init, marking_graph, places),
            svg: generate_svg(m_names, marking_graph, transitions)?,
            png: generate_png(m_names, marking_graph, transitions)?,
        })
    }

    pub fn save_smv(&self, smv_file_path: &str) -> Result<(), io::Error> {
        fs::write(smv_file_path, &self.smv)
    }

    pub fn save_svg(&self, svg_file_path: &str) -> Result<(), io::Error> {
        fs::write(svg_file_path, &self.svg)
    }

    pub fn save_png(&self, svg_file_path: &str) -> Result<(), io::Error> {
        fs::write(svg_file_path, &self.png)
    }
}

pub fn vector_to_string(v: &Vec<Option<i32>>, sep: &str) -> String {
    v.iter()
        .map(|x| match x {
            Some(x) => x.to_string(),
            None => "n".to_string(),
        })
        .collect::<Vec<_>>()
        .join(sep)
}

pub fn generate_smv_code(
    m_init: &Vec<Option<i32>>,
    marking_graph: &HashMap<Vec<Option<i32>>, Vec<(Vec<i32>, Vec<Option<i32>>)>>,
    places: &Vec<Place>,
) -> String {
    CODE_TEMPLATE
        .replace(
            "STATES",
            &format!(
                "{{{}}}",
                marking_graph
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
            &marking_graph
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
                        marking_graph
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
        )
}

fn generate_dot_template(
    m_names: &Vec<String>,
    marking_graph: &HashMap<Vec<Option<i32>>, Vec<(Vec<i32>, Vec<Option<i32>>)>>,
    transitions: &Vec<Vec<(i32, i32)>>,
) -> String {
    DOT_TEMPLATE
        .replace("NAMING", &format!("\"{}\"", m_names.join("-")))
        .replace(
            "GRAPH",
            &marking_graph
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
        )
        .to_string()
}

pub fn generate_svg(
    m_names: &Vec<String>,
    marking_graph: &HashMap<Vec<Option<i32>>, Vec<(Vec<i32>, Vec<Option<i32>>)>>,
    transitions: &Vec<Vec<(i32, i32)>>,
) -> Result<String, anyhow::Error> {
    let dot_template = generate_dot_template(m_names, marking_graph, transitions);

    let graph = graphviz_rust::parse(&dot_template)
        .map_err(|e| ErrorTypes::CannotAssembleGraph { reason: e })?;
    Ok(graphviz_rust::exec(
        graph,
        &mut PrinterContext::default(),
        vec![Format::Svg.into()],
    )?)
}

pub fn generate_png(
    m_names: &Vec<String>,
    marking_graph: &HashMap<Vec<Option<i32>>, Vec<(Vec<i32>, Vec<Option<i32>>)>>,
    transitions: &Vec<Vec<(i32, i32)>>,
) -> Result<Vec<u8>, anyhow::Error> {
    let dot_template = generate_dot_template(m_names, marking_graph, transitions);
    let echo_child = Command::new("echo")
        .arg(format!("{}", dot_template))
        .stdout(Stdio::piped())
        .spawn()?;
    let dot_child = Command::new("dot")
        .arg("-Tpng")
        .stdin(Stdio::from(echo_child.stdout.unwrap()))
        .output()?;
    let file = dot_child.stdout;
    Ok(file)
}
