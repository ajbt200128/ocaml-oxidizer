use clap::Parser;
use std::process::ExitCode;

/// Run an OCaml script in an embedded, self-contained bytecode interpreter.
#[derive(Parser)]
#[command(name = "ocaml-oxidizer", version)]
struct Cli {
    /// Path to the .ml file to run.
    file: std::path::PathBuf,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let src = match std::fs::read_to_string(&cli.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ocaml-oxidizer: cannot read {}: {e}", cli.file.display());
            return ExitCode::FAILURE;
        }
    };

    let interp = ocaml_oxidizer::Interp::new();
    match interp.run(&src) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("ocaml-oxidizer: {e:?}");
            ExitCode::FAILURE
        }
    }
}
