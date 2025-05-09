use clap::Parser;
use error_stack::Result;
use poem_admin::{
    cli,
    errors::Error,
    utils::{styled::styled, version::version},
};

#[derive(Debug, clap::Parser)]
#[command(name = "percas", version, long_version = version(), styles=styled())]
struct Command {
    #[clap(subcommand)]
    cmd: SubCommand,
}

impl Command {
    pub fn run(self) -> Result<(), Error> {
        match self.cmd {
            SubCommand::Server(cmd) => cmd.run(),
            SubCommand::InitData(cmd) => cmd.run(),
        }
    }
}

#[derive(Debug, clap::Subcommand)]
enum SubCommand {
    Server(cli::CommandStart),
    InitData(cli::CommandInitData),
}

fn main() -> Result<(), Error> {
    let cmd = Command::parse();
    cmd.run()
}
