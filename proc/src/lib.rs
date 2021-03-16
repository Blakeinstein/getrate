use rillrate::{RillRate, Table};
use sysinfo::{System, SystemExt, Process, ProcessExt};
use std::{error::Error, u64};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::collections::BTreeSet;
use std::thread;
use std::time::{ SystemTime, Duration };

fn set_info(os_table: &Table, process: &Process, id: u64) {
    let row_id = id.into();
    os_table.set_cell(row_id, 1.into(), process.name(), Some(SystemTime::now()));
    os_table.set_cell(row_id, 2.into(), format!("{:.4}%", process.cpu_usage()), Some(SystemTime::now()));
    os_table.set_cell(row_id, 3.into(), format!("{:.3}", process.memory() as f32 / 1000.), Some(SystemTime::now()));
    os_table.set_cell(row_id, 4.into(), format!("{:.3} / {:.3}", process.disk_usage().total_read_bytes as f32 /1000000., process.disk_usage().total_written_bytes/1000000), Some(SystemTime::now()));
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut sys = System::new();
    let _rillrate = RillRate::from_env("osmon")?;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;
    {
        let os_table = Table::create("processes")?;
        let mut proc_set = BTreeSet::new();
        sys.refresh_all();
        
        os_table.add_col(0.into(), Some("PID".into()));
        os_table.add_col(1.into(), Some("Process Name".into()));
        os_table.add_col(2.into(), Some("Cpu Usage".into()));
        os_table.add_col(3.into(), Some("Memory Usage (Mb)".into()));
        os_table.add_col(4.into(), Some("Disk Usage (Mb) R/W".into()));
        for (id, process) in sys.get_processes() {
            let row_id = (*id as u64).into();
            proc_set.insert(*id);
            os_table.add_row(row_id, Some(id.to_string()));
            os_table.set_cell(row_id, 0.into(), id, Some(SystemTime::now()));
            set_info(&os_table, &process, *id as u64);
        }
        println!("OsMon running on http://localhost:9090");
        while running.load(Ordering::SeqCst) {
            sys.refresh_processes();
            let mut temp_set = BTreeSet::new();
            for (id, process) in sys.get_processes() {
                temp_set.insert(*id);
                let row_id = (*id as u64).into();
                if !proc_set.contains(id) {
                    os_table.add_row(row_id, Some(id.to_string()));
                }
                set_info(&os_table, &process, *id as u64);
            }
            for id in proc_set.difference(&temp_set) {
                os_table.del_row((*id as u64).into());
            }
            proc_set = temp_set;
            thread::sleep(Duration::from_millis(100));
        }
    }
    Ok(())
}