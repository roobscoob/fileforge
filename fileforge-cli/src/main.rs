use clap::{command, Command};

fn main() {
  let matches = command!()
    .subcommand(Command::new("npdm").about("Handling NPDMs"))
    .get_matches();

  if let Some(matches) = matches.subcommand_matches("npdm") {
    println!("We're doing NPDM shit today!")
  }
}
