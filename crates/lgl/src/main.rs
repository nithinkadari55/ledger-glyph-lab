use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use lgl::world::{LabWorld, WorldOptions};

#[derive(Parser)]
#[command(
    name = "lgl",
    version,
    about = "Compile Typst and report layout diagnostics as JSON lines"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Check(CheckArgs),
    Fix(FixArgs),
    /// Print the rule catalog as JSON.
    Rules,
}

#[derive(clap::Args)]
struct FixArgs {
    #[command(flatten)]
    check: CheckArgs,
    #[arg(long)]
    out: PathBuf,
    #[arg(long, default_value_t = 5)]
    max_rounds: usize,
}

#[derive(clap::Args)]
struct CheckArgs {
    input: PathBuf,
    #[arg(long)]
    root: Option<PathBuf>,
    #[arg(long = "font-path")]
    font_paths: Vec<PathBuf>,
    #[arg(long)]
    package_path: Option<PathBuf>,
    #[arg(long)]
    package_cache_path: Option<PathBuf>,
    #[arg(long, env = "SOURCE_DATE_EPOCH")]
    creation_timestamp: Option<i64>,
    #[arg(long)]
    pdf: Option<PathBuf>,
    #[arg(long)]
    geometry: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Check(args) => match check(args) {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::from(1),
            Err(message) => {
                eprintln!("lgl: {message}");
                ExitCode::from(2)
            }
        },
        Command::Rules => {
            let catalog = serde_json::json!({
                "rules": lgl::catalog::CATALOG,
                "typst": lgl::catalog::TYPST_RULES,
                "intent": { "allow-overlap": "label content with <allow-overlap> to exempt it from L006" },
            });
            println!("{}", serde_json::to_string_pretty(&catalog).unwrap());
            ExitCode::SUCCESS
        }
        Command::Fix(args) => match fix(args) {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::from(1),
            Err(message) => {
                eprintln!("lgl: {message}");
                ExitCode::from(2)
            }
        },
    }
}

fn check(args: CheckArgs) -> Result<bool, String> {
    let options = WorldOptions {
        root: args.root,
        font_dirs: args.font_paths,
        package_path: args.package_path,
        package_cache_path: args.package_cache_path,
        creation_timestamp: args.creation_timestamp,
    };
    let world = LabWorld::new(&args.input, &options).map_err(|err| err.to_string())?;
    let outcome = lgl::check(&world, args.pdf.is_some());

    let stdout = io::stdout();
    let mut out = stdout.lock();
    for diagnostic in &outcome.diagnostics {
        let line = serde_json::to_string(diagnostic).map_err(|err| err.to_string())?;
        writeln!(out, "{line}").map_err(|err| err.to_string())?;
    }

    if let (Some(path), Some(geometry)) = (&args.geometry, &outcome.geometry) {
        let json = serde_json::to_vec_pretty(geometry).map_err(|err| err.to_string())?;
        std::fs::write(path, json).map_err(|err| format!("{}: {err}", path.display()))?;
    }

    let passed = outcome.passed();
    if let (Some(path), Some(pdf)) = (&args.pdf, &outcome.pdf)
        && passed
    {
        std::fs::write(path, pdf).map_err(|err| format!("{}: {err}", path.display()))?;
    }

    Ok(passed)
}

fn fix(args: FixArgs) -> Result<bool, String> {
    let input = &args.check.input;
    let out = &args.out;
    if input.parent() != out.parent() {
        return Err("--out must be in the same directory as the input".into());
    }
    let text = std::fs::read_to_string(input)
        .map_err(|err| format!("{}: {err}", input.display()))?;

    let options = WorldOptions {
        root: args.check.root.clone(),
        font_dirs: args.check.font_paths.clone(),
        package_path: args.check.package_path.clone(),
        package_cache_path: args.check.package_cache_path.clone(),
        creation_timestamp: args.check.creation_timestamp,
    };

    let (applied_total, rounds) = lgl::repair(&text, out, &options, args.max_rounds)?;
    eprintln!(
        "lgl: applied {applied_total} edit(s) in {rounds} round(s) to {}",
        out.display()
    );

    check(CheckArgs { input: out.clone(), ..args.check })
}
