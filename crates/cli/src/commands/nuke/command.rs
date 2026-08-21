use std::io::{self, Write};

use anyhow::Result;
use local::ZygoLocalService;

pub fn nuke_database() -> Result<()> {
    print!("Delete the Zygo local database? [y/n] ");
    io::stdout().flush()?;

    let mut response = String::new();
    loop {
        response.clear();
        io::stdin().read_line(&mut response)?;

        match response.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => {
                if ZygoLocalService::delete_database()? {
                    println!("Zygo local database deleted.");
                } else {
                    println!("No Zygo local database was found.");
                }
                return Ok(());
            }
            "n" | "no" | "" => {
                println!("Cancelled.");
                return Ok(());
            }
            _ => {
                print!("Please enter y or n: ");
                io::stdout().flush()?;
            }
        }
    }
}
