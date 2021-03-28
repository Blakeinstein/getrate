#[allow(unused_imports)]
use rillrate::Table;
use sysinfo::{System as Sys, SystemExt, Process, ProcessExt};
use anyhow::Error;
use std::{error, usize};
use std::collections::BTreeSet;
use std::time::{ SystemTime, Duration };
use meio::{Actor, Context, InterruptedBy, StartedBy, System, task::*};
use async_trait::async_trait;

struct ProcessWatcher {
    sys: Sys,
    os_table: Table,
    proc_set: BTreeSet<usize>,
}

impl ProcessWatcher {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        let sys = Sys::new();
        let os_table = Table::create("processes")?;
        let proc_set = BTreeSet::new();     
        os_table.add_col(0.into(), Some("PID".into()));
        os_table.add_col(1.into(), Some("Process Name".into()));
        os_table.add_col(2.into(), Some("Status".into()));
        os_table.add_col(3.into(), Some("Cpu Usage".into()));
        os_table.add_col(4.into(), Some("Memory Usage (Mb)".into()));
        os_table.add_col(5.into(), Some("Disk Usage (Mb) R/W".into()));
        Ok(Self {
            sys,
            os_table,
            proc_set,
        })
    }

    fn set_info(&self, process: &Process, id: u64) {
        let row_id = id.into();
        let timestamp = Some(SystemTime::now());
        self.os_table.set_cell(row_id, 1.into(), process.name(), timestamp);
        self.os_table.set_cell(row_id, 2.into(), process.status().to_string(), timestamp);
        self.os_table.set_cell(row_id, 3.into(), format!("{:.4}%", process.cpu_usage()), timestamp);
        self.os_table.set_cell(row_id, 4.into(), format!("{:.3}", process.memory() as f32 / 1000.), timestamp);
        self.os_table.set_cell(row_id, 5.into(), format!("{:.3} / {:.3}", process.disk_usage().total_read_bytes as f32 /1000000., process.disk_usage().total_written_bytes/1000000), timestamp);
    }
}

impl Actor for ProcessWatcher {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<System> for ProcessWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.sys.refresh_all();
        for (id, process) in self.sys.get_processes() {
            let row_id = (*id as u64).into();
            self.proc_set.insert(*id as usize);
            if process.name().is_empty() {
                continue;
            }
            self.os_table.add_row(row_id, Some(id.to_string()));
            self.os_table.set_cell(row_id, 0.into(), id, Some(SystemTime::now()));
            self.set_info(&process, *id as u64);
        }
        let task = HeartBeat::new(Duration::from_millis(100), ctx.address().clone());
        ctx.spawn_task(task, ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for ProcessWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl OnTick for ProcessWatcher {
    async fn tick(&mut self, _: Tick, _: &mut meio::Context<Self>) -> Result<(), Error> {
        self.sys.refresh_processes();
        let mut temp_set = BTreeSet::new();
        for (id, process) in self.sys.get_processes() {
            temp_set.insert(*id as usize);
            let row_id = (*id as u64).into();
            if process.name().is_empty() {
                if self.proc_set.contains(&(*id as usize)) {
                    self.os_table.del_row((*id as u64).into());
                }
                continue;
            }
            if !self.proc_set.contains(&(*id as usize)) {
                self.os_table.add_row(row_id, Some(id.to_string()));
            }
            self.set_info(&process, *id as u64);
        }
        for id in self.proc_set.difference(&temp_set) {
            self.os_table.del_row((*id as u64).into());
        }
        self.proc_set = temp_set;
        Ok(())
    }

    async fn done(&mut self, ctx: &mut meio::Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::try_init()?;
    let proc = ProcessWatcher::new()?;
    let osmon = System::spawn(proc);
    
    System::wait_or_interrupt(osmon).await?;
    Ok(())
}