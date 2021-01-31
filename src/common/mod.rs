
pub enum Event {
    Shutdown,
    AllOk,
}

pub fn fatal_error(message: &str) -> ! {
    eprintln!("{}ERROR: {}", termion::color::Fg(termion::color::Red), message);
    std::process::exit(1);
}