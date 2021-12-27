extern crate clap;
extern crate console_subscriber;
extern crate tokio;
extern crate tracing;
extern crate tracing_subscriber;

use std::time::Duration;

use clap::StructOpt;
use tracing::Instrument;

mod application;

#[tokio::main]
async fn main() {
    application::init_tracing();

    let args = application::Args::parse();

    tracing::info!("{:#?}", args);

    // Test that the console is working currently
    let tasks = (0..5)
        .map(|i| {
            tokio::task::Builder::new()
                .name(&format!("Task {}", i))
                .spawn(
                    async move {
                        for j in 0..10 {
                            tracing::warn!("Task {} Iter {}", i, j);
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                    .instrument(tracing::info_span!("Printing Numbers")),
                )
        })
        .collect::<Vec<_>>();

    // wait for them all to complete
    for task in tasks.into_iter() {
        let _ = task.await;
    }
}
