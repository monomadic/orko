extern crate pickle;

pub fn main() {
    pickle::command::run_docopt().expect("success");
}
