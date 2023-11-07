use std::fs::File;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::ng::{compute_transitions_map, generate_name, read_map_from_file, read_map_from_resource};

pub mod ng;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate(Generate),
    Compute(Compute),
}

#[derive(Args)]
struct Generate {
    #[arg(long, short = 'i')]
    input: Option<PathBuf>,
    #[arg(long, short = 'n', default_value = "1")]
    number: i32,
}

#[derive(Args)]
struct Compute {
    #[arg(long, short = 'i')]
    input: PathBuf,
    #[arg(long, short = 'o')]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = include_str!("../firstnames_en.map");
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate(generate) => {
            let transition_map = match &generate.input {
                None => read_map_from_resource(bytes)?,
                Some(path) => read_map_from_file(path)?
            };

            for _ in 0..generate.number {
                let name = generate_name(&transition_map);
                println!("{}", name);
            }
        }
        Commands::Compute(compute) => {
            let transition_map = compute_transitions_map(&compute.input)?;
            let file = File::create(&compute.output).unwrap();
            serde_json::to_writer(file, &transition_map)?;
        }
    }

    Ok(())
}
