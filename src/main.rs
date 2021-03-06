#[macro_use]
extern crate clap;
#[macro_use]
extern crate prettytable;

pub mod database;
pub mod generator;
pub mod clipboard;
pub mod parser;


use std::env;
use std::fs;

use clap::{App, Arg, SubCommand};
use log::*;

fn main() {
    let mut builder = env_logger::Builder::from_default_env();
    builder.format_level(false).format_module_path(false).format_timestamp(None);
    match env::var("RUST_LOG") {
        Ok(_) => builder.init(),
        Err(_) => {
            env::set_var("RUST_LOG", "spacebar=info");
            builder.init();
            debug!("Defaulting to info level logging."); //Schrodinger's dead code.
        },
    }

    let matches = App::new("Spacebar Tagging System")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("init")
                    .about("Initialize a new spacebar database in the current directory"))
        .subcommand(SubCommand::with_name("new")
                    .about("Create a new spacebar")
                    .arg(Arg::with_name("name")
                         .short("n")
                         .long("name")
                         .value_name("STRING")
                         .help("A name for the new spacebar")
                         .required(true)
                         .takes_value(true))
                    .arg(Arg::with_name("description")
                         .short("d")
                         .long("description")
                         .value_name("STRING")
                         .help("A description of the new spacebar")
                         .required(false)
                         .takes_value(true))
        )
        .subcommand(SubCommand::with_name("search")
                    .about("Search for a spacebar")
                    .arg(Arg::with_name("clipboard")
                         .short("c")
                         .long("clipboard")
                         .help("Search the system clipboard for a spacebar")
                         .required(false)
                         .takes_value(false))
                    .arg(Arg::with_name("web")
                         .short("w")
                         .long("website")
                         .value_name("URL")
                         .help("Scrape the page of the given URL for a spacebar")
                         .required(false)
                         .takes_value(true))
                    .arg(Arg::with_name("file")
                         .short("f")
                         .long("file")
                         .value_name("FILE_PATH")
                         .help("Search the given file for a spacebar")
                         .required(false)
                         .takes_value(true))
        )
        .subcommand(SubCommand::with_name("show")
                    .about("Show spacebars in database")
                    .help("Show spacebars in database"))
        .get_matches();

    if let Some(_) = matches.subcommand_matches("init") {
        match fs::create_dir("./.sbdb/") {
            Ok(_) => {
                match database::connect("./.sbdb/spacebars.db") {
                    Some(_) => {
                        info!("Initialized successfully.");
                        std::process::exit(0);
                    },
                    None => {
                        error!("Failed to initialize.");
                        std::process::exit(1);
                    }
                }
            },
            Err(e) => {
                error!("Failed to initialize: {}", e);
                std::process::exit(1);
            }
        }
    }

    let conn = match database::connect("./.sbdb/spacebars.db") {
        Some(s) => s,
        None => std::process::exit(1),
    };

    if let Some(matches_new) = matches.subcommand_matches("new") {
        if matches_new.is_present("name") {
            let spacebar: generator::Spacebar;
            if matches_new.is_present("description") {
                spacebar = generator::generate_spacebar(matches_new.value_of("name").unwrap().to_string(), Some(matches_new.value_of("description").unwrap().to_string()));
            } else {
                spacebar = generator::generate_spacebar(matches_new.value_of("name").unwrap().to_string(), None);
            }
            database::insert_spacebar(&conn, spacebar);
        } else {
            error!("You must specify a name for the new spacebar.");
        }
    }

    if let Some(matches_search) = matches.subcommand_matches("search") {
        if matches_search.is_present("clipboard") {
            match parser::parse_clipboard() {
                Some(o) => match database::select_spacebar(&conn, o) {
                    Some(s) => parser::print_spacebar(s),
                    None => println!("Found a spacebar, but it wasn't in the database. It could be someone elses' (spooky)"),
                },
                None => info!("Could not find spacebar in the clipboard"),
            }
        }
        if matches_search.is_present("web") {
            match matches_search.value_of("web") {
                Some(s) => {
                    match parser::parse_web(s) {
                        Some(o) => match database::select_spacebar(&conn, o) {
                            Some(s) => parser::print_spacebar(s),
                            None => println!("Found a spacebar, but it wasn't in the database. It could be someone elses' (spooky)"),
                        },
                        None => info!("Could not find spacebar at: {}", s),
                    }
                },
                None => info!("No url provided."),
            }
        }
        if matches_search.is_present("file") {
            match matches_search.value_of("file") {
                Some(s) => {
                    match parser::parse_file(s) {
                        Some(o) => match database::select_spacebar(&conn, o) {
                            Some(s) => parser::print_spacebar(s),
                            None => println!("Found a spacebar, but it wasn't in the database. It could be someone elses' (spooky)"),
                        },
                        None => info!("Could not find spacebar in file: {}", s),
                    }
                },
                None => info!("No file path provided."),
            }
        }
    }

    if let Some(_) = matches.subcommand_matches("show") {
        parser::print_spacebars(database::show_spacebars(&conn));
    }
}
