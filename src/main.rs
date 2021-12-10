use clap::{ Arg, App, AppSettings, SubCommand };

use lattice_lib::*;

fn main() -> Result<(), Error> {
    let matches = App::new("Lattice Programming Language")
        .version("v0.1")
        .about("A stack-based language with a cell-based memory structure.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("sim")
            .about("Simulate the program without compiling.")
            .arg(Arg::from_usage("[FILE]")
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("com")
            .about("Compile the program.")
            .arg(Arg::with_name("run")
                 .short("r")
                 .help("run after compiling")
            )
            .arg(Arg::from_usage("[FILE]")
                .required(true)
            )
        ).get_matches();

    if let Some(matches) = matches.subcommand_matches("com") {
        let file = matches.value_of("FILE").unwrap();
        let lines = load_file(file)?;
        let tokens = lex_lines(lines)?;
        
        com::compile(&tokens, &file, matches.is_present("run"))?;
    } else if let Some(matches) = matches.subcommand_matches("sim") {
        let file = matches.value_of("FILE").unwrap();
        let lines = load_file(file)?;
        let tokens = lex_lines(lines)?;
        
        sim::simulate(&tokens)?;
    } else {
        unreachable!()
    }

    Ok(())
}
