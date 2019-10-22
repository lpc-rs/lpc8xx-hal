use std::fs;

use termion::{
    color,
    style,
};


fn main() {
    let openocd_cfg = match (cfg!(feature = "82x"), cfg!(feature = "845")) {
        (true, false) => "openocd_82x.cfg",
        (false, true) => "openocd_84x.cfg",

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

    fs::copy(openocd_cfg, "target/openocd.cfg")
        .expect("Failed to copy OpenOCD configuration");
}
