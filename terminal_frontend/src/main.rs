use backend::error_type::ErrorTypes;
use backend::graph_gen::*;
use backend::petri_parser::parser::*;
use clap::*;
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
