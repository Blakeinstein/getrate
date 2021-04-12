use proc;
use docker;
use std::error::Error;
use rillrate::RillRate;
use env_logger;

fn main() -> Result<(), Box<dyn Error>>{
    env_logger::try_init()?;
    let _rillrate = RillRate::from_env("osmon")?;
    // proc::main()?;
    docker::main()?;
    Ok(())
}
