use rillrate::Table;
use anyhow::Error;
use meio::{Actor, Context, InterruptedBy, StartedBy, System, task::*};
use async_trait::async_trait;
use std::time::SystemTime;
use shiplift::Docker;
use shiplift::rep::Container;

struct DockerWatcher{
    docker: Docker,
    docker_table: Table
}

impl DockerWatcher {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        let docker = Docker::new();
        let docker_table = Table::create("Containers");
        docker_table.add_col(0.into(), Some("created".into()));
        docker_table.add_col(1.into(), Some("command".into()));
        docker_table.add_col(2.into(), Some("id".into()));
        docker_table.add_col(3.into(), Some("image".into()));
        docker_table.add_col(4.into(), Some("labels".into()));
        docker_table.add_col(5.into(), Some("names".into()));
        docker_table.add_col(6.into(), Some("ports".into()));
        docker_table.add_col(7.into(), Some("state".into()));
        docker_table.add_col(8.into(), Some("status".into()));
        docker_table.add_col(9.into(), Some("size_rw".into()));
        Ok(Self{
            docker,
            docker_table
        })
    }

    fn set_info(&self, container: &Container, id: u64) {
        let row_id = id.into();
        let timestamp = Some(SystemTime::now());
        self.docker_table.set_cell(row_id, 0.into(), container.created.to_rfc2822(), timestamp);
        self.docker_table.set_cell(row_id, 1.into(), container.command, timestamp);
        self.docker_table.set_cell(row_id, 2.into(), container.id, timestamp);
        self.docker_table.set_cell(row_id, 3.into(), container.image, timestamp);
        self.docker_table.set_cell(row_id, 4.into(), container.labels, timestamp);
        self.docker_table.set_cell(row_id, 5.into(), container.names.join(","), timestamp);
        self.docker_table.set_cell(row_id, 6.into(), container.ports, timestamp);
        self.docker_table.set_cell(row_id, 7.into(), container.state, timestamp);
        self.docker_table.set_cell(row_id, 8.into(), container.status, timestamp);
        self.docker_table.set_cell(row_id, 9.into(), container.size_rw.unwrap_or(0).to_string(), timestamp);
    }
}

impl Actor for DockerWatcher {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<System> for DockerWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        for (id, container) in self.docker.containers().list(&Default::default()).await?.enumerate() {
            self.set_info(container, id);
        };
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for DockerWatcher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::try_init()?;
    let docker = DockerWatcher::new()?;
    let docker_mon = System::spawn(docker);
    
    System::wait_or_interrupt(docker_mon).await?;
    Ok(())
}