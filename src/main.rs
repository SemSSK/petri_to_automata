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
mod output_generators;
mod petri_parser;

use crate::graph_gen::*;
use clap::*;
use error_type::ErrorTypes;
use petri_parser::parser::*;
use std::fs;

#[derive(Debug, Parser)]
/// Program that allows to convert a petri network to a Finite state automata
/// using json to represent petri network and smv to represent the automata
#[command(author, version)]
pub struct Args {
    /// path to the source of the petri network
    #[arg(short,long,default_value_t=String::from("./net.petri"))]
    source: String,
    /// path to the output file
    #[arg(short,long,default_value_t=String::from("./automata"))]
    output: String,
}

fn main() -> Result<(), anyhow::Error> {
    // READING INPUTS
    let args = Args::parse();
    let petri = fs::read_to_string(&args.source)?;
    let input = if petri.starts_with("{") {
        serde_json::from_str(&petri)?
    } else {
        PetriNet::new(&petri)?.generate_input()
    };

    if input
        .transitions
        .iter()
        .any(|t| t.len() != input.m_init.len())
    {
        return Err(anyhow::Error::new(ErrorTypes::TransitionSizeNotMatching {
            expected: input.m_init.len(),
        }));
    }

    let output = compile_to_output(input)?;

    output.save_smv(&format!("{}{}", args.output, ".smv"))?;
    output.save_svg(&format!("{}{}", args.output, ".svg"))?;

    open::that(&format!("{}{}", args.output, ".svg"))?;
    Ok(())
}
