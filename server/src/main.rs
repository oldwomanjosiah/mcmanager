extern crate anyhow;
extern crate clap;
extern crate console_subscriber;
extern crate futures;
extern crate tokio;
extern crate tracing;
extern crate tracing_subscriber;
#[macro_use]
extern crate async_stream;

use std::pin::Pin;

use clap::StructOpt;
use futures::Stream;
use information::SystemInfo;
use tonic::{Response, Status};
use tracing::{debug_span, info, info_span, warn_span};
use tracing_futures::Instrument;

mod application;
mod information;
mod prelude;
mod util;

use prelude::*;

use tonic::transport::NamedService;

mod hello_world {
    use crate::{information::SystemInfo, prelude::*};

    tonic::include_proto!("helloworld");

    pub struct HelloWorldServiceImpl {
        pub sysinfo: SystemInfo,
    }

    #[tonic::async_trait]
    impl hello_world_service_server::HelloWorldService for HelloWorldServiceImpl {
        async fn hello_world(
            &self,
            request: tonic::Request<HelloRequest>,
        ) -> Result<tonic::Response<HelloResponse>, tonic::Status> {
            let name = request.into_inner().name;
            let greeting = format!("{:#?}", self.sysinfo.borrow());

            tracing::info!("Responding to {}", name);

            Ok(HelloResponse { greeting }.as_msg())
        }
    }
}

mod events {
    mod proto {
        tonic::include_proto!("event");
    }

    use futures::Stream;
    use tokio_stream::StreamExt;
    // Re-Exports
    pub use proto::events_server::EventsServer;
    pub use proto::Event;
    pub use proto::SystemSnapshot;

    use proto::event::Event as EventInner;
    use proto::EventSubscription;
    use tonic::Code;
    use tonic::Request;
    use tonic::Response;
    use tonic::Status;
    use tracing::log::info;

    use crate::information::SystemInfo;
    use crate::prelude::*;

    pub struct EventsService {
        pub system_info: SystemInfo,
    }

    #[tonic::async_trait]
    impl proto::events_server::Events for EventsService {
        type SubscribeStream = StreamDescriptor<Event>;

        async fn subscribe(&self, request: Request<EventSubscription>) -> StreamResponse<Event> {
            let system_info = self.system_info.clone();

            Ok(stream! {
                match request.remote_addr() {
                    Some(addr) => info!("Event Service Starting Stream to {addr}"),
                    None => info!("Event Service Starting Stream to {{unknown}}"),
                }

                let mut system_info = system_info.collect();

                loop {
                    let info = system_info.next().await;
                    yield match info {
                        Some(info) => Ok(Event {
                            event: Some(EventInner::SystemSnapshot(info)),
                        }),
                        None => Err(Status::new(Code::Ok, "System Information Collector Was Cancelled")),
                    }
                }
            }
            .as_msg())
        }

        async fn snapshot(
            &self,
            request: Request<EventSubscription>,
        ) -> Result<Response<Event>, Status> {
            Ok(Event {
                event: Some(EventInner::SystemSnapshot(
                    self.system_info.borrow().clone(),
                )),
            }
            .as_msg())
        }
    }
}

#[tracing::instrument(skip(sysinfo))]
async fn launch_services(sysinfo: SystemInfo) -> Result<()> {
    tracing::info!("Starting gRPC Server at 0.0.0.0:50051");

    tonic::transport::Server::builder()
        .concurrency_limit_per_connection(32)
        .add_service(
            events::EventsServer::new(events::EventsService {
                system_info: sysinfo.clone(),
            })
            .accept_gzip()
            .send_gzip(),
        )
        .add_service(
            hello_world::hello_world_service_server::HelloWorldServiceServer::new(
                hello_world::HelloWorldServiceImpl { sysinfo },
            )
            .accept_gzip()
            .send_gzip(),
        )
        .serve("0.0.0.0:50051".parse()?)
        .await
        .context("Running Tonic Unauthenticated Server")
}

#[tokio::main]
async fn main() -> Result<()> {
    application::init_tracing();

    let args = application::Args::parse();

    tracing::info!("{args:#?}");

    let rx = information::start_sysinfo();

    tokio::task::Builder::new()
        .name("gRPC Server")
        .spawn(launch_services(rx.clone()))
        .await??;

    info!("Ended with value: {:#?}", rx.borrow());

    Ok(())
}
