use templar_lib;

use std;
use std::path::{Path, PathBuf};

struct TemplarDirectiveHandler {
    pub current_directory: PathBuf,
    pub destination_directory: PathBuf,
}

pub fn compile_templar(base_directory:&Path, source:&Path, destination:&Path) -> Result<(), BuildErrorReason> {
    let directive_handler = TemplarDirectiveHandler { current_directory: base_directory.to_path_buf(), destination_directory: destination.to_path_buf() };

    let nodes = parse_template(source)?;
    let out_path = destination.with_extension("html");
    let mut file = fs::File::create(out_path)?;

    let empty_context = TemplateContext::empty();

    templar::output::write_out(nodes.as_slice(), &empty_context, &mut file, 0, 2, &directive_handler)?;
    file.sync_all()?;

    Ok(())
}
