use std::error::Error;

use local::{database_path, db::migrate};
use turso::Builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let path = database_path()?;
    let path_string = path.to_string_lossy().into_owned();

    let database = Builder::new_local(&path_string).build().await?;
    let mut connection = database.connect()?;
    migrate(&mut connection).await?;

    let mut rows = connection.query("SELECT 1", ()).await?;
    let row = rows.next().await?.ok_or("SELECT 1 returned no rows")?;
    let value: i64 = row.get(0)?;

    assert_eq!(value, 1);
    println!("local database ready at {}", path.display());
    println!("SELECT 1 returned {value}");

    Ok(())
}
