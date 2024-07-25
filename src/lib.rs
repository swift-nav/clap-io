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

//! Add optional `--input` and `--output` flags to a clap command. If `--input` is not specified,
//! it defaults to (locked) stdin. If `--output` is not specified, it defaults to (locked) stdout.
//!
//! # Examples
//!
//! Add get `--input` and `--output` flags to your program:
//!
//! ```rust,no_run
//! use clap::Parser;
//! use clap_io::InputOutput;
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[clap(flatten)]
//!     io: InputOutput,
//! }
//!
//! let cli = Cli::parse();
//! let mut input = cli.io.input.open().unwrap();
//! let mut output = cli.io.output.open().unwrap();
//! std::io::copy(&mut input, &mut output).unwrap();
//! ```
//!
//! Add just one:
//!
//! ```rust,no_run
//! use clap::Parser;
//! use clap_io::Input;
//!
//! #[derive(Parser)]
//! struct Cli {
//!    #[clap(long = "in")]
//!     input: Input,
//! }
//!
//! let cli = Cli::parse();
//! eprintln!("is tty? {}", cli.input.is_tty());
//! eprintln!("path? {:?}", cli.input.path());
//! ```

use std::{
    ffi::{OsStr, OsString},
    fmt,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{Args, ValueHint};

const STDIO: &str = "-";
const STDIN: &str = "<stdin>";
const STDOUT: &str = "<stdout>";

/// Combined input and output options.
#[derive(Debug, Args)]
pub struct InputOutput {
    /// Input file path
    #[arg(
        long,
        default_value_os_t,
        value_hint = ValueHint::FilePath,
    )]
    pub input: Input,

    /// Output file path
    #[arg(
        long,
        default_value_os_t,
        value_hint = ValueHint::FilePath,
    )]
    pub output: Output,
}

/// Either a file or stdin.
#[derive(Debug, Clone)]
pub struct Input(Stream);

impl Input {
    /// Open the input stream.
    pub fn open(self) -> io::Result<Box<dyn Read + 'static>> {
        match self.0 {
            Stream::File(_) => {
                let file = self.open_file().unwrap()?;
                Ok(Box::new(file))
            }
            Stream::Stdin { .. } => {
                let stdin = self.open_stdin().unwrap();
                Ok(Box::new(stdin))
            }
            Stream::Stdout { .. } => unreachable!("stdout is an output"),
        }
    }

    /// Open the input as stdin.
    pub fn open_stdin(self) -> Result<io::StdinLock<'static>, Self> {
        match self.0 {
            Stream::Stdin { .. } => {
                let stdin = Box::leak(Box::new(io::stdin()));
                Ok(stdin.lock())
            }
            _ => Err(self),
        }
    }

    /// Open the input as a file.
    pub fn open_file(&self) -> Option<io::Result<File>> {
        match &self.0 {
            Stream::File(path) => match File::open(&path) {
                Ok(file) => Some(Ok(file)),
                Err(e) => Some(Err(io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to open input file `{}`. Cause: {}",
                        path.display(),
                        e
                    ),
                ))),
            },
            _ => None,
        }
    }

    /// Is this input a TTY?
    pub fn is_tty(&self) -> bool {
        self.0.is_tty()
    }

    /// If the input is a file get the path.
    pub fn path(&self) -> Option<&Path> {
        self.0.path()
    }
}

impl Default for Input {
    fn default() -> Self {
        Self(Stream::stdin())
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Input {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s.as_ref()))
    }
}

impl From<&OsStr> for Input {
    fn from(s: &OsStr) -> Self {
        if s == STDIO || s == STDIN {
            Self(Stream::stdin())
        } else {
            Self(Stream::file(s))
        }
    }
}

impl From<Input> for OsString {
    fn from(input: Input) -> Self {
        input.0.into()
    }
}

/// Either a file or stdout.
#[derive(Debug, Clone)]
pub struct Output(Stream);

impl Output {
    /// Open the output stream.
    pub fn open(self) -> io::Result<Box<dyn Write + 'static>> {
        match self.0 {
            Stream::File(_) => {
                let file = self.open_file().unwrap()?;
                Ok(Box::new(file))
            }
            Stream::Stdout { .. } => {
                let stdout = self.open_stdout().unwrap();
                Ok(Box::new(stdout))
            }
            Stream::Stdin { .. } => unreachable!("stdin is an input"),
        }
    }

    /// Open the output as stdout.
    pub fn open_stdout(self) -> Result<io::StdoutLock<'static>, Self> {
        match self.0 {
            Stream::Stdout { .. } => {
                let stdout = Box::leak(Box::new(io::stdout()));
                Ok(stdout.lock())
            }
            _ => Err(self),
        }
    }

    /// Open the output as a file.
    pub fn open_file(&self) -> Option<io::Result<File>> {
        match &self.0 {
            Stream::File(path) => match File::create(&path) {
                Ok(file) => Some(Ok(file)),
                Err(e) => Some(Err(io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to open output file `{}`. Cause: {}",
                        path.display(),
                        e
                    ),
                ))),
            },
            _ => None,
        }
    }

    /// Is this output a TTY?
    pub fn is_tty(&self) -> bool {
        self.0.is_tty()
    }

    /// If the output is a file get the path.
    pub fn path(&self) -> Option<&Path> {
        self.0.path()
    }
}

impl Default for Output {
    fn default() -> Self {
        Self(Stream::stdout())
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Output {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s.as_ref()))
    }
}

impl From<&OsStr> for Output {
    fn from(s: &OsStr) -> Self {
        if s == STDIO || s == STDOUT {
            Self(Stream::stdout())
        } else {
            Self(Stream::file(s))
        }
    }
}

impl From<Output> for OsString {
    fn from(output: Output) -> Self {
        output.0.into()
    }
}

#[derive(Debug, Clone)]
enum Stream {
    File(PathBuf),
    Stdin { tty: bool },
    Stdout { tty: bool },
}

impl Stream {
    fn file(path: &OsStr) -> Self {
        Self::File(path.into())
    }

    fn stdin() -> Self {
        Self::Stdin {
            tty: atty::is(atty::Stream::Stdin),
        }
    }

    fn stdout() -> Self {
        Self::Stdout {
            tty: atty::is(atty::Stream::Stdout),
        }
    }

    fn is_tty(&self) -> bool {
        matches!(self, Self::Stdin { tty } | Self::Stdout { tty } if *tty)
    }

    fn path(&self) -> Option<&Path> {
        if let Self::File(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => path.display().fmt(f),
            Self::Stdin { .. } => STDIN.fmt(f),
            Self::Stdout { .. } => STDOUT.fmt(f),
        }
    }
}

impl From<Stream> for OsString {
    fn from(s: Stream) -> OsString {
        match s {
            Stream::File(path) => path.into(),
            Stream::Stdin { .. } => STDIN.into(),
            Stream::Stdout { .. } => STDOUT.into(),
        }
    }
}
