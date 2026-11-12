fn main() {
    if let Err(error) = vfa_desktop::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
