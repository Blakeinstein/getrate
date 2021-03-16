use proc;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>>{
    proc::main()?;
    Ok(())
}
