use std::io::Write;

pub enum PageAction {
    Next,
    Quit,
}

pub fn prompt_next_page(nc: bool) -> PageAction {
    let esc = if nc { "\x1b[1m" } else { "\x1b[1;36m" };
    print!("\n{esc}[SPACE] next page  [Q] quit\x1b[0m ");
    std::io::stdout().flush().ok();
    let action = read_single_key();
    println!();
    action
}

#[cfg(unix)]
fn read_single_key() -> PageAction {
    use std::io::Read;
    use std::os::unix::io::AsRawFd;

    let tty = match std::fs::OpenOptions::new().read(true).open("/dev/tty") {
        Ok(f) => f,
        Err(_) => return PageAction::Quit,
    };
    let fd = tty.as_raw_fd();

    let old = unsafe {
        let mut t = std::mem::MaybeUninit::<libc::termios>::uninit();
        libc::tcgetattr(fd, t.as_mut_ptr());
        t.assume_init()
    };

    let mut raw = old;
    raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    raw.c_cc[libc::VMIN] = 1;
    raw.c_cc[libc::VTIME] = 0;
    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw); }

    let mut buf = [0u8; 1];
    let mut file = tty;
    let _ = file.read(&mut buf);

    unsafe { libc::tcsetattr(fd, libc::TCSANOW, &old); }

    match buf[0] {
        b' ' => PageAction::Next,
        _ => PageAction::Quit,
    }
}

#[cfg(not(unix))]
fn read_single_key() -> PageAction {
    PageAction::Quit
}
