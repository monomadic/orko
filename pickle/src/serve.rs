use std::path::{PathBuf};
use std::net::SocketAddr;

use iron::Iron;
use staticfile::Static;
use mount::Mount;

#[derive(Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    pub root_dir: PathBuf,
}

pub enum Error {
    // Io(io::Error),
    // AddrParse(std::net::AddrParseError),
    // Std(Box<StdError + Send + Sync>),
    // ParseInt(std::num::ParseIntError),
}

pub fn serve(config: ServerConfig) -> Result<(), Error> {
    let mut mount = Mount::new();
    mount.mount("/", Static::new(config.root_dir));
    Iron::new(mount).http(config.addr).unwrap();

    Ok(())
}
