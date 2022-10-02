use clap::{Args, Parser};
use error_stack::Report;
use thiserror::Error;

use self::{download::Download, upload::Upload};

mod download;
mod upload;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) enum Command {
    Greet(Greet),
    Download(Download),
    Upload(Upload),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct Greet {
    /// Name of the person to greet
    #[arg(short, long)]
    pub(crate) name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    pub(crate) count: u8,
}

impl Greet {
    pub(crate) fn run(self) -> Result<(), Report<GreetError>> {
        if self.count > 10 {
            return Err(Report::new(GreetError::TooManyGreetings));
        }
        for _ in 0..self.count {
            println!("Hello {}!", self.name);
        }
        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum GreetError {
    #[error("Too many greetings were requested; at most 9 greetings can be provided")]
    TooManyGreetings,
}
