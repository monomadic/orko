use std::io;
use std::env;
use std::thread;

use docopt::Docopt;
use watch;
use colored::Colorize;

use build;
use serve;
use output;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const SERVER_ADDRESS: &'static str = "127.0.0.1:9000";

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
            output::print_summary(&source_directory, build_result);

            if args.get_bool("serve") {
                let l = format!("\nServing {} at http://{}\n", site, SERVER_ADDRESS);
                println!("{}", l.cyan());

                let server_root = target_directory.clone();
                let _ = thread::spawn(|| {
                    let _ = serve::serve(serve::ServerConfig {
                        addr: SERVER_ADDRESS.parse().unwrap(),
                        root_dir: server_root,
                    });
                });

                let watcher = watch::watch(&source_directory);
                'fs: loop {
                    match watcher.change_events.recv() {
                        Ok(watch::ChangeEvent{ path, op:_, cookie:_ }) => {
                            if let Some(_) = path {
                                let build_result = build::build(&source_directory, &target_directory);
                                output::print_summary(&source_directory, build_result);
                            }
                        },
                        Err(_) => break 'fs,
                    }
                }
            }
        }
    }

    Ok(())
}
