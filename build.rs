use std::{
    env,
    fs::{self, File},
    io::{self, prelude::*},
    path::PathBuf,
};

use termion::{color, style};

fn main() -> Result<(), Error> {
    let target = Target::read();

    copy_openocd_config(target)?;
    copy_memory_config(target)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

/// Make target-specific configuration available where OpenOCD expects it
fn copy_openocd_config(target: Target) -> Result<(), io::Error> {
    let openocd_cfg = match target.family {
        Family::LPC82x => &include_bytes!("openocd_82x.cfg")[..],
        Family::LPC84x => &include_bytes!("openocd_84x.cfg")[..],
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

/// Make `memory.x` available to dependent crates
fn copy_memory_config(target: Target) -> Result<(), Error> {
    let memory_x = match target.sub_family {
        SubFamily::LPC822 => include_bytes!("memory_16_4.x").as_ref(),
        SubFamily::LPC824 => include_bytes!("memory_32_8.x").as_ref(),
        SubFamily::LPC845 => include_bytes!("memory_64_16.x").as_ref(),
    };

    let out_dir = env::var("OUT_DIR")?;
    let out_dir = PathBuf::from(out_dir);

    File::create(out_dir.join("memory.x"))?.write_all(memory_x)?;

    // Tell Cargo where to find the file.
    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=memory_16_4.x");
    println!("cargo:rerun-if-changed=memory_32_8.x");
    println!("cargo:rerun-if-changed=memory_64_16.x");

    Ok(())
}

#[derive(Clone, Copy)]
struct Target {
    family: Family,
    sub_family: SubFamily,
}

impl Target {
    fn read() -> Self {
        let (family, sub_family) = Family::read();

        Self { family, sub_family }
    }
}

#[derive(Clone, Copy)]
enum Family {
    LPC82x,
    LPC84x,
}

impl Family {
    fn read() -> (Self, SubFamily) {
        let f82x = cfg!(feature = "82x");

        let s822 = cfg!(feature = "822");
        let s824 = cfg!(feature = "824");
        let s845 = cfg!(feature = "845");

        match (f82x, s822, s824, s845) {
            (true, false, false, false) => {
                warn_unspecific_selection();
                (Family::LPC82x, SubFamily::LPC822)
            }
            (true, true, false, false) => {
                (Family::LPC82x, SubFamily::LPC822)
            }
            (true, false, true, false) => {
                (Family::LPC82x, SubFamily::LPC824)
            }
            (false, false, false, true) => {
                (Family::LPC84x, SubFamily::LPC845)
            }

            (false, false, false, false) => {
                error("You must select a target.

If you added LPC8xx HAL as a dependency to your crate, you can select a target by enabling the respective feature in `Cargo.toml`.

If you're running an example from the repository, select a target by passing the desired target as a command-line argument, for example `--features=824m201jhi33`.


Please refer to the documentation for more details."
                )
            }
            _ => {
                error(
                    "You can not select more than one target."
                )
            }
        }
    }
}

#[derive(Clone, Copy)]
enum SubFamily {
    LPC822,
    LPC824,
    LPC845,
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

fn error(message: &str) -> ! {
    panic!(
        "\n\n\n{}{}{}{}{}\n\n\n",
        style::Bold,
        color::Fg(color::Red),
        message,
        color::Fg(color::Reset),
        style::Reset,
    );
}

fn warn_unspecific_selection() {
    if !cfg!(feature = "no-target-warning") {
        println!(
            "cargo:warning=You have selected a family (e.g. LPC82x), but not a specific target within that family. Your application will only be able to use the hardware resources available on all targets of that family, while your specific target might have more peripherals or memory available.",
        );
    }
}
