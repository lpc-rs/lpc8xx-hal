use termion::{
    color,
    style,
};


fn main() {
    match (cfg!(feature = "82x"), cfg!(feature = "845")) {
        (true, false) => (),
        (false, true) => (),

        _ => {
            panic!(
                "\n\n\n{}{}You must select exactly one target platform. Pass `--features=82x` or `--features=845`{}{}\n\n\n",
                style::Bold,
                color::Fg(color::Red),
                color::Fg(color::Reset),
                style::Reset,
            );
        }
    }
}
