mod fishing;
mod gathering;

use std::env;

use fishing::start_fishing;
use gathering::start_gathering;

fn main() {
    println!("-- HoloCure need to be in windowed mode 1920x1080. --");

    let fishing = env::args().any(|a| a == "--fishing" || a == "-f");
    let gathering = env::args().any(|a| a == "--gathering" || a == "-g");

    if fishing {
        start_fishing();
    }

    if gathering {
        start_gathering();
    }

    eprintln!("Choose either --fishing or --gathering");
}
