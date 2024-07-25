// Copyright (c) 2023 Swift Navigation
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use anyhow::Result;

use clap::Parser;

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
