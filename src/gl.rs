fn getline() -> Option<String> {
    use termion::event::Key;
    use termion::input::TermRead;
    let mut curin = String::new();
    let stdin = std::io::stdin();

    for c in stdin.keys() {
        match c.ok()? {
            Key::Char('\n') => break,
            Key::Ctrl('d') => {
                if curin.is_empty() {
                    return None;
                } else {
                    break;
                }
            }
            Key::Ctrl('c') => {
                return if curin.is_empty() {
                    None
                } else {
                    Some(String::new())
                }
            }
            Key::Char(c) => curin.push(c),
            _ => {}
        }
    }

    Some(curin)
}

pub fn getline_wprompt(prompt: &str) -> Option<String> {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    write!(stdout, "{}", prompt).ok()?;
    stdout.flush().ok()?;
    getline()
}
