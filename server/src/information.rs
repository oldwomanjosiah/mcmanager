//! Runtime Information Gathering

use std::{future::Future, time::Duration};

use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};
use tokio::{
    select,
    sync::watch::{self, channel, Receiver, Sender},
    time::sleep,
};
use tracing::{debug, info, info_span, instrument::Instrumented, trace, Instrument};

pub type SystemInfo = Receiver<SystemSnapshot>;

#[derive(Clone, Copy, Debug)]
pub struct SystemSnapshot {
    timestamp: u64,
    cpu_pressure: f64,
    mem_pressure: f32,
}

impl SystemSnapshot {
    pub(self) fn new() -> Self {
        Self {
            timestamp: 0,
            cpu_pressure: 0.0,
            mem_pressure: 0.0,
        }
    }
}

/// Start a task to update a [`SystemSnapshot`] and a receiver that allows you to check the current
/// values
pub fn start_sysinfo() -> SystemInfo {
    let (tx, rx) = channel(SystemSnapshot::new());

    tokio::task::Builder::new()
        .name("Sysinfo watch")
        .spawn(system_information_task(tx));

    return rx; // .instrument(info_span!("Checking System Info"));
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

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Unexpected Time from Before Unix Epoch")
            .as_secs();

        tokio::task::block_in_place(|| {
            system.refresh_all();
        });

        //let cpu_pressure: f32 = system.processors().iter().map(|it| it.cpu_usage()).sum();
        let cpu_pressure = system.load_average().one;
        let mem_pressure: f32 = system.used_memory() as f32 / system.total_memory() as f32;

        let snapshot = SystemSnapshot {
            timestamp,
            cpu_pressure,
            mem_pressure,
        };

        debug!("Sysinfo snapshot: {:#?}", snapshot);

        match tx.send(snapshot) {
            Err(_) => {
                info!("Last Receiver Closed for sysinfo while calculating, stopping");
                break;
            }
            _ => (),
        }
    }
}
