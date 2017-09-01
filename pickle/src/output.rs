use std::io;
use build::*;
use std::path::Path;
use colored::Colorize;
use pad::PadStr;

//ScanDirectory,
//Copy(PathBuf),
//Ignore,
//Compile { extension: String, destination: PathBuf },

pub fn print_summary(path:&Path, result: io::Result<Vec<ProcessedFile>>) {
    let l = format!("\nBuilding {}\n", path.to_str().unwrap());
    println!("{}", l.cyan());

    match result {
        Ok(files) => {
            let contains_errors = files.iter().any(|f| f.result.is_err());

            let use_files = if contains_errors {
                files.into_iter().filter(|f| f.result.is_err()).collect()
            } else {
                files
            };


            for file in use_files {
                let color = match file.action {
                    BuildAction::Skip => "magenta",
                    BuildAction::Ignore => "yellow",
                    _ => if file.result.is_ok() { "green" } else { "red" }
                };
                let line = format!("{:?} - {:?}", file.source, file.action);
                println!("{}", line.color(color));

                if let Some(err) = file.result.err() {
                    match err {
                        BuildErrorReason::IO(io) => {
                            let line = format!("IO error {:?}", io).red();
                            println!("{}\n", line);
                        },
                        BuildErrorReason::Sass(sass_reason) => {
                            let line = format!("Sass compilation error {:?}", sass_reason).red();
                            println!("{}\n", line);
                        },
                        BuildErrorReason::TemplarParse(parse_error) => {
                            println!("Problem compiling templar template:");
                            for (idx, c) in parse_error.context.iter().enumerate() {
                                let line_number = parse_error.line_number + 2 + idx - parse_error.context.len();
                                let padded_line_number = format!("{}:", line_number).pad_to_width(5);
                                let line = format!("{} {}", padded_line_number, c);
                                println!("{}", line);
                            }
                            println!("reason -> {:?}\n", parse_error.reason);
                        },
                        BuildErrorReason::TemplarWrite(write_error) => {
                            let line = format!("Templar Write Error {:?}", write_error).red();
                            println!("{}\n", line);
                        },
                        BuildErrorReason::UTF8Error(utf8_error) => {
                            let line = format!("File was not UTF8 {:?}", utf8_error).red();
                            println!("{}\n", line);
                        },
                    }
                }


            }
        }
        Err(io_error) => {
            let line = format!("io error -> {:?}", io_error);
            println!("{}", line.red());
        }
    }
}