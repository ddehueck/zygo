use zygo_core::dirs::Directories;

fn main() {
    let dirs = Directories::new().unwrap();
    match std::env::args().nth(1).as_deref() {
        Some("db-path") => println!("{}", dirs.store_db_path().display()),
        Some("store-root") => println!("{}", dirs.store_root().display()),
        _ => eprintln!("Usage: zygo-cli <db-path|store-root>"),
    }
}

