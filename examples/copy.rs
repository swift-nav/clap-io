use anyhow::Result;

use clap::Parser;
use clap_io::InputOutput;

/// Copy --input to --output
#[derive(Debug, Parser)]
struct Cli {
    #[clap(flatten)]
    io: InputOutput,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    eprintln!("reading from {:?}", cli.io.input);
    eprintln!("writing to {:?}", cli.io.output);
    let mut input = cli.io.input.open()?;
    let mut output = cli.io.output.open()?;
    std::io::copy(&mut input, &mut output)?;
    Ok(())
}
