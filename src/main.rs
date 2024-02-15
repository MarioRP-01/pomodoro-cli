use std::thread;
use std::time::Duration;

fn main() {
    let mut seconds: u8 = 0;
    loop {
        println!("{}", seconds);
        thread::sleep(Duration::from_secs(1));
        seconds = (seconds + 1) % 60;
    }
}
