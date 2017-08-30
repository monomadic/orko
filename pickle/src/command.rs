use std::io;
use std::env;

use docopt::Docopt;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const USAGE: &'static str = "
Pickels~! 🥒

Usage:
  pickle serve
  pickle serve <name>
  pickle build
  pickle build <name>
  pickle (-h | --help)
  pickle --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

pub fn run_docopt() -> io::Result<()> {
    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt
            .version(Some(VERSION.to_string()))
            .parse())
        .unwrap_or_else(|e| e.exit());

    if args.get_bool("serve") {
        let site = args.get_vec("<name>")[0];
        let pwd = env::current_dir()?;

        let source_directory = pwd.clone().push(site);

        let mut target_directory = pwd.clone();
        target_directory.push("_build");
        target_directory.push(site);

        // let build_result = build::build(&source, &dest);


    }
    Ok(())
}
