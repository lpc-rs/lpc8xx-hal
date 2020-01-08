use std::{
    env,
    fs::{self, File},
    io::{self, prelude::*},
    path::PathBuf,
};

use termion::{color, style};

fn main() -> Result<(), Error> {
    copy_openocd_config()?;
    copy_memory_config()?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

fn copy_openocd_config() -> Result<(), io::Error> {
    let openocd_cfg = match (cfg!(feature = "82x"), cfg!(feature = "845")) {
        (true, false) => &include_bytes!("openocd_82x.cfg")[..],
        (false, true) => &include_bytes!("openocd_84x.cfg")[..],

        _ => {
            panic!(
                "\n\n\n{}{}You must select exactly one target platform. Pass `--features=82x` or `--features=845`{}{}\n\n\n",
                style::Bold,
                color::Fg(color::Red),
                color::Fg(color::Reset),
                style::Reset,
            );
        }
    };

    // These file operations are not using the `OUT_DIR` environment variable on
    // purpose. `OUT_DIR` points to a directory within target, whose path even
    // contains a hash. This configuration file needs to be referenced from the
    // GDB configuration, which can't just ask Cargo where to look for it.
    fs::create_dir_all("target")?;
    File::create("target/openocd.cfg")?.write_all(openocd_cfg)?;

    println!("cargo:rerun-if-changed=openocd_82x.cfg");
    println!("cargo:rerun-if-changed=openocd_84x.cfg");

    Ok(())
}

fn copy_memory_config() -> Result<(), Error> {
    let out_dir = env::var("OUT_DIR")?;
    let out_dir = PathBuf::from(out_dir);

    File::create(out_dir.join("memory.x"))?
        .write_all(include_bytes!("memory.x"))?;

    // Tell Cargo where to find the file.
    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=memory.x");

    Ok(())
}

#[derive(Debug)]
enum Error {
    Env(env::VarError),
    Io(io::Error),
}

impl From<env::VarError> for Error {
    fn from(error: env::VarError) -> Self {
        Self::Env(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
