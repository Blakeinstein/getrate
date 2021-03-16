use rillrate::RillRate;
use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    // let _rillrate = RillRate::from_env("osmon")?;
    // let os_table = Table::create("os.process.table")?;
    // my_table.add_col(0.into(), Some("Thread".into()));
    // my_table.add_col(1.into(), Some("State".into()));

    // tbl.add_row(i.into(), Some(tname.clone()));
    println!("Hello from proc");
    Ok(())
}