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

    match result {
        Ok(files) => {
            let contains_errors = files.iter().any(|f| f.result.is_err());

            let use_files = if contains_errors {
                files.into_iter().filter(|f| f.result.is_err()).collect()
            } else {
                files
            };

            for file in use_files {
                // println!("{:?}", file);
                // let color = match file.action {
                //     BuildAction::Skip => "magenta",
                //     BuildAction::Ignore => "yellow",
                //     _ => if file.result.is_ok() { "green" } else { "red" }
                // };
                // let line = format!("{:?} - {:?}", file.source, file.action);
                // println!("{}", line.color(color));

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
                            println!("Templar compilation error:");
                            for (idx, c) in parse_error.context.iter().enumerate() {
                                let line_number = parse_error.line_number + 2 + idx - parse_error.context.len();
                                let padded_line_number = format!("{}:", line_number).pad_to_width(5);

                                let marker = if parse_error.line_number == line_number {
                                    ">"
                                } else { " " };

                                let line = format!("{}{} {}", marker, padded_line_number, c);
                                println!("{}", line);
                            }

                            println!("reason -> {:?}\n", parse_error.reason);
                        },
                        BuildErrorReason::TemplarWrite(write_error) => {
                            match write_error {
                                ::templar::output::WriteError::DirectiveError(e) => {

                                    let error_message = format!("Templar error:\n  {}\n  Command: ={}\n  Reason: {}", file.source.into_os_string().into_string().unwrap(), e.directive, e.reason).red();
                                    println!("{}\n", error_message);
                                },
                                ::templar::output::WriteError::IO(_) => {}
                            }
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
            let line = format!("{}", io_error);
            println!("{}", line.red());
        }
    }
}