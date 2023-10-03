use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "wf")]
#[command(about = "A tool to automate some common dev tasks", long_about = None)]
pub struct WfArgs {
    #[command(subcommand)]
    pub command: WfCommands,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum WfCommands {
    /// Initialize workflow app settings
    Init,
    /// Testing subcommands
    #[command(subcommand, hide = true)]
    Test(WfTestCommands),
    /// Start a new workflow in the current repository
    Start {
        #[arg(required = true, help = "Ticket id to create the branch from")]
        ticket_id: String,
    },
    /// Push current work branch to remote repository
    Push,
    /// Do nothing, just to test
    Noop,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct WfTestArgs {
    #[command(subcommand)]
    sub_command: Option<WfTestCommands>,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum WfTestCommands {
    /// Sub 1
    Sub1,
    /// Sub 2
    Sub2,
    /// All
    All,
}
