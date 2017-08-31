use std::io;
use std::env;
use std::thread;

use docopt::Docopt;

use build;
use serve;

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

    if args.get_bool("serve") || args.get_bool("build") {
        let name = args.get_vec("<name>");
        if name.is_empty() {
            println!("multi site not supported yet");
        } else {
            let site = args.get_vec("<name>")[0];
            let pwd = env::current_dir()?;

            let mut source_directory = pwd.clone();
            source_directory.push(site);

            let mut target_directory = pwd.clone();
            target_directory.push("_build");
            target_directory.push(site);

            let build_result = build::build(&source_directory, &target_directory);

            if args.get_bool("serve") {
                println!("single: {:?}", site);

                let server_root = target_directory.clone();
                let _ = thread::spawn(|| {
                    let _ = serve::serve(serve::ServerConfig {
                        addr: "127.0.0.1:3000".parse().unwrap(),
                        root_dir: server_root,
                    });
                });
            }

            // loop { }

            match build_result {
                Ok(_) => println!("done."),
                Err(e) => println!("error processing {:?}: {:?}", source_directory, e.kind()),
            }
        }
    }

    Ok(())
}
