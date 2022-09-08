use anyhow::Result;

use clap::Parser;
use clap_io::{Input, Output};

/// Copy --in to --out
#[derive(Debug, Parser)]
struct Cli {
    #[clap(long = "in", parse(from_os_str))]
    input: Input,

    #[clap(long = "out", parse(from_os_str))]
    output: Output,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    eprintln!("reading from {:?}", cli.input);
    eprintln!("writing to {:?}", cli.output);
    let mut input = cli.input.open()?;
    let mut output = cli.output.open()?;
    std::io::copy(&mut input, &mut output)?;
    Ok(())
}
