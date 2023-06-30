use std::collections::HashMap;

use crate::graph_gen::Place;

// const DOT_TEMPLATE: &str = r#"
//     digraph {
//         NAMING
//         GRAPH
//     }
// "#;

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