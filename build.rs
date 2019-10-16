fn main() {
    match (cfg!(feature = "82x"), cfg!(feature = "845")) {
        (true, false) => (),
        (false, true) => (),

        _ => {
            panic!("\n\n\nYou must select exactly one target platform. Pass `--features=82x` or `--features=845`\n\n\n");
        }
    }
}
