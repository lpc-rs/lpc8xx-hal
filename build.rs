use std::{
    io::prelude::*,
    env,
    fs::File,
    path::PathBuf,
};

use termion::{
    color,
    style,
};


fn main() {
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

    let out_dir = env::var_os("OUT_DIR")
        .unwrap_or("target".into());
    let out_dir = &PathBuf::from(out_dir);

    File::create(out_dir.join("openocd.cfg"))
        .expect("Failed to create openocd.cfg")
        .write_all(openocd_cfg)
        .expect("Failed to write openocd.cfg");
}
