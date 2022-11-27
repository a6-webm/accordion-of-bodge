use wayland_client::Display;

fn main() {
    let display: Display = Display::connect_to_env()
        .expect("Could not connect to wayland environment");
    loop {
        
    }
}
