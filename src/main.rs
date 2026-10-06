use std::{mem::MaybeUninit, thread, time::Duration};

use crate::double_buffer::DoubleBuffer;

mod double_buffer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let millis: u64 = if args.is_empty() {
        1000
    } else if args.len() == 2 {
        parse_u64(args[1]
            .clone())
            .expect("Couldn't parse given argument to milliseconds!")
    } else {
        panic!("Wrong amount of arguments passed!");
    };

    let (term_width, term_height) = terminal_size();
    let mut dbuf = DoubleBuffer::new(term_width, term_height);

    loop {
        print!("{}", dbuf.render_string());
        dbuf.update();
        dbuf.swap();
        thread::sleep(Duration::from_millis(millis));
    }
}

fn terminal_size() -> (u16, u16) {
    let mut winsz = MaybeUninit::<libc::winsize>::uninit();
    let result = unsafe {
        libc::ioctl(libc::STDIN_FILENO, libc::TIOCGWINSZ, winsz.as_mut_ptr())
    };

    if result == -1 {
        panic!("Couldn't read terminal dimensions!");
    }

    let winsz = unsafe { winsz.assume_init() };
    if winsz.ws_row <= 0 || winsz.ws_col <= 0 {
        panic!("Couldn't read proper terminal dimensions!");
    }

    (winsz.ws_col, winsz.ws_col)
}

fn parse_u64(s: String) -> Option<u64> {
    let mut cur: u64 = 0;
    for ch in s.chars() {
        match ch {
            ch if ch.is_ascii_digit() => cur = cur * 10 + u64::from(ch),
            _ => return None,
        }
    }

    Some(cur)
}
