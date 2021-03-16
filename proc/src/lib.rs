use rillrate::{RillRate, Table};
use sysinfo::{System, SystemExt, ProcessExt};
use std::{error::Error, u64};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{ SystemTime, Duration };

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut sys = System::new();
    let _rillrate = RillRate::from_env("osmon")?;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;
    {
        let os_table = Table::create("os.process.table")?;
        sys.refresh_processes();
        
        os_table.add_col(0.into(), Some("PID".into()));
        os_table.add_col(1.into(), Some("Process Name".into()));
        os_table.add_col(2.into(), Some("Cpu Usage".into()));
        os_table.add_col(3.into(), Some("Memory Usage".into()));
        os_table.add_col(4.into(), Some("Disk Usage".into()));
        for (id, process) in sys.get_processes() {
            let row_id = (*id as u64).into();
            os_table.add_row(row_id, Some(id.to_string()));
            os_table.set_cell(row_id, 0.into(), id, Some(SystemTime::now()));
            os_table.set_cell(row_id, 1.into(), process.name(), Some(SystemTime::now()));
            os_table.set_cell(row_id, 2.into(), process.cpu_usage(), Some(SystemTime::now()));
            os_table.set_cell(row_id, 3.into(), process.memory(), Some(SystemTime::now()));
            os_table.set_cell(row_id, 4.into(), process.disk_usage().total_read_bytes, Some(SystemTime::now()));
        }
        println!("OsMon running on http://localhost:9090");
        while running.load(Ordering::SeqCst) {
            //TODO
            continue;
        }
    }
    Ok(())
}