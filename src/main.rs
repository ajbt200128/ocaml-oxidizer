use clap::Parser;
use std::process::ExitCode;

mod update;

/// Run an OCaml script in an embedded, self-contained bytecode interpreter.
#[derive(Parser)]
#[command(name = "ox", version)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[command(flatten)]
    run: RunArgs,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Update ox in place to the latest release.
    Update,
}

#[derive(clap::Args)]
struct RunArgs {
    /// Type-check the script without running it.
    #[arg(long)]
    check: bool,
    /// Path to the .ml file.
    file: Option<std::path::PathBuf>,
    /// Arguments passed to the script as Sys.argv.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Update) => match update::run() {
            Ok(msg) => {
                println!("{msg}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("ox update: {e}");
                ExitCode::FAILURE
            }
        },
        None => run_script(cli.run),
    }
}

fn run_script(run: RunArgs) -> ExitCode {
    let Some(file) = run.file else {
        eprintln!("ox: no script given (try `ox <file.ml>` or `ox update`)");
        return ExitCode::FAILURE;
    };

    let src = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ox: cannot read {}: {e}", file.display());
            return ExitCode::FAILURE;
        }
    };

    let name = file.display().to_string();
    let mut argv = vec![name.clone()];
    argv.extend(run.args.iter().cloned());

    let interp = ocaml_oxidizer::Interp::with_args(&argv);
    let result = if run.check {
        interp.check(&name, &src)
    } else {
        interp.run(&name, &src)
    };

    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("ox: {e:?}");
            ExitCode::FAILURE
        }
    }
}
