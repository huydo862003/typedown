use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "typedown", about = "Typedown CLI")]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {}

fn main() -> anyhow::Result<()> {
  #[allow(unreachable_code)]
  let _cli = Cli::parse();
  Ok(())
}
