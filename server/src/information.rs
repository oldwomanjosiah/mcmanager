//! Runtime Information Gathering

use std::time::Duration;

use sysinfo::{RefreshKind, System, SystemExt};
use tokio::{
    select,
    sync::watch::{channel, Receiver, Sender},
    time::sleep,
};
use tracing::{debug, info, trace};

use data::events::SystemSnapshot;
pub type SystemInfo = Receiver<SystemSnapshot>;

/// Start a task to update a [`SystemSnapshot`] and a receiver that allows you to check the current
/// values
pub fn start_sysinfo() -> SystemInfo {
    let (tx, rx) = channel(SystemSnapshot::default());

    tokio::task::Builder::new()
        .name("Sysinfo watch")
        .spawn(system_information_task(tx));

    rx // .instrument(info_span!("Checking System Info"));
}

async fn system_information_task(tx: Sender<SystemSnapshot>) {
    debug!("Setting up Sysinfo");

    let mut system = System::new_with_specifics(RefreshKind::new().with_cpu().with_memory());

    trace!("Sysinfo Loop Starting");
    loop {
        select! {
            _ = tx.closed() => {
                info!("Last Receiver Closed for sysinfo before calculating, stopping");
                break;
            },
            _ = sleep(Duration::from_secs(3)) => {},
        }

        let unixtime = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Unexpected Time from Before Unix Epoch")
            .as_secs();

        tokio::task::block_in_place(|| {
            system.refresh_all();
        });

        //let cpu_pressure: f32 = system.processors().iter().map(|it| it.cpu_usage()).sum();
        let cpu_pressure = system.load_average().one as _;
        let mem_pressure: f32 = system.used_memory() as f32 / system.total_memory() as f32;

        let snapshot = SystemSnapshot {
            unixtime,
            cpu_pressure,
            mem_pressure,
        };

        debug!("Sysinfo snapshot: {:#?}", snapshot);

        if tx.send(snapshot).is_err() {
            info!("Last Receiver Closed for sysinfo while calculating, stopping");
            break;
        }
    }
}
