use std::io;

/// Wait for user to type in "yes"
fn wait_for_start() {
    let mut buffer = String::new();
    while buffer != "yes" {
        println!("Type 'yes' to start game:");
        let _ = io::stdin().read_line(&mut buffer);
    }
}

fn main() {
    wait_for_start();
}
