use clap::Parser;
use std::process::ExitCode;

/// Run an OCaml script in an embedded, self-contained bytecode interpreter.
#[derive(Parser)]
#[command(name = "ox", version)]
struct Cli {
    /// Type-check the script without running it.
    #[arg(long)]
    check: bool,
    /// Path to the .ml file.
    file: std::path::PathBuf,
    /// Arguments passed to the script as Sys.argv.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let src = match std::fs::read_to_string(&cli.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ox: cannot read {}: {e}", cli.file.display());
            return ExitCode::FAILURE;
        }
    };

    let name = cli.file.display().to_string();
    let mut argv = vec![name.clone()];
    argv.extend(cli.args.iter().cloned());

    let interp = ocaml_oxidizer::Interp::with_args(&argv);
    let result = if cli.check {
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
