use anyhow::Result;

use clap::{Parser, CommandFactory};
use clap_io::{Input, Output};

macro_rules! converter {
    ($name:ident) => {
        #[derive(Parser)]
        #[clap(name = stringify!($name), bin_name = stringify!($name))]
        struct Cli {
            input: clap_io::Input,
            output: clap_io::Output,
        }
    };
}

converter!(copy);

// /// Copy <input> to <output>
// #[derive(Debug, Parser)]
// struct Cli {
// input: Input,
// output: Output,
// }

fn main() -> Result<()> {
    let cli = Cli::parse();
    // let cmd = Cli::command();
    // cmd.name("whatever");
    // cmd.bin_name("whatever");
    // <cmd as Parser>::parse_from()
    eprintln!("reading from {:#?}", cli.input);
    eprintln!("writing to {:#?}", cli.output);
    let mut input = cli.input.open()?;
    let mut output = cli.output.open()?;
    std::io::copy(&mut input, &mut output)?;
    Ok(())
}
