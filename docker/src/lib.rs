#[tokio::main]
pub async fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::try_init()?;
    let proc = ProcessWatcher::new()?;
    let osmon = System::spawn(proc);
    
    System::wait_or_interrupt(osmon).await?;
    Ok(())
}