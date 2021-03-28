use proc;
use std::error::Error;
use rillrate::RillRate;

fn main() -> Result<(), Box<dyn Error>>{
    let _rillrate = RillRate::from_env("osmon")?;
    proc::main()?;
    Ok(())
}
