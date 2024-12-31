use clap::{command, Parser, Subcommand};
use anyhow::{anyhow, Result};
use log::debug;

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct CliParams {
    #[command(subcommand)]
    command: Command
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum Command {
    Daemon {
        #[command(subcommand)]
        sub: DaemonCommand
    },
    Launch {
        #[command(subcommand)]
        sub: LaunchCommand
    },
    Operate {
        #[command(subcommand)]
        sub: OpCommand
    },
    #[cfg(feature="debug")]
    Debug
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum DaemonCommand {
    Start {
        #[arg(short, long)]
        force: bool
    },
    Stop
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum LaunchCommand {
    App,
    // Window,
    WindowInApp
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum OpCommand {
    MoveCursor {
        #[command(subcommand)]
        direction: Direction
    },
    ShowWindows,
    SelectCurrent,
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum Direction {
    Prev,
    Next
}

pub(crate) trait CliHandler: Sized {
    type Output;

    fn start_daemon(force: bool) -> Self::Output;

    fn stop_daemon() -> Self::Output;

    fn launch(sc: LaunchCommand) -> Self::Output;

    fn move_cursor(direction: Direction) -> Self::Output;

    fn show_windows() -> Self::Output;

    fn select_current() -> Self::Output;
}

pub(crate) fn run_cli<T: CliHandler>() -> Result<T::Output> {
    let params = CliParams::try_parse()
        .map_err(|e| anyhow!(e))?;
    debug!("Given command: {params:?}");

    let result = match params.command {
        Command::Daemon { sub } => match sub {
            DaemonCommand::Start { force } => T::start_daemon(force),
            DaemonCommand::Stop => T::stop_daemon(),
        },
        Command::Launch { sub } => T::launch(sub),
        Command::Operate { sub } => match sub {
            OpCommand::MoveCursor { direction } => T::move_cursor(direction),
            OpCommand::ShowWindows => T::show_windows(),
            OpCommand::SelectCurrent => T::select_current(),
        },
        #[cfg(feature="debug")]
        Command::Debug { sub } => todo!(),
    };

    Ok(result)
}
