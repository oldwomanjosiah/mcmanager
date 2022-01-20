extern crate anyhow;
extern crate clap;
extern crate console_subscriber;
extern crate futures;
extern crate tokio;
extern crate tracing;
extern crate tracing_subscriber;
#[macro_use]
extern crate async_stream;

use std::path::PathBuf;

use application::Configuration;
use auth::manager::{AuthManager, AuthManagerConfig};
use clap::StructOpt;

use data::IntoServer;
use information::SystemInfo;
use tracing::info;

mod application;
mod information;
mod prelude;
mod util;

use prelude::*;

use crate::application::SubCommand;

mod hello_world {
    use crate::information::SystemInfo;
    use crate::prelude::*;
    use data::hello::*;

    pub struct HelloWorldImpl {
        pub sysinfo: SystemInfo,
    }

    #[tonic::async_trait]
    impl HelloWorld for HelloWorldImpl {
        async fn hello_world(
            &self,
            request: tonic::Request<HelloRequest>,
        ) -> Result<tonic::Response<HelloResponse>, tonic::Status> {
            let name = request.into_inner().name;
            let greeting = format!("{:#?}", self.sysinfo.borrow());

            tracing::info!("Responding to {}", name);

            Ok(HelloResponse { greeting }.into_msg())
        }
    }
}

mod events {
    use crate::{information::SystemInfo, prelude::*};
    use data::events::*;
    use tonic::{Code, Request, Response, Status};
    use tracing::info;

    pub struct EventsService {
        pub system_info: SystemInfo,
    }

    #[tonic::async_trait]
    impl Events for EventsService {
        type SubscribeStream = StreamDescriptor<EventResponse>;

        async fn subscribe(
            &self,
            request: Request<EventSubscription>,
        ) -> StreamResponse<EventResponse> {
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
                        Some(info) => Ok(EventResponse {
                            event: Some(Event::SystemSnapshot(info)),
                        }),
                        None => Err(Status::new(Code::Ok, "System Information Collector Was Cancelled")),
                    }
                }
            }
            .into_msg())
        }

        async fn snapshot(
            &self,
            _request: Request<EventSubscription>,
        ) -> Result<Response<EventResponse>, Status> {
            Ok(EventResponse {
                event: Some(Event::SystemSnapshot(self.system_info.borrow().clone())),
            }
            .into_msg())
        }
    }
}

#[tracing::instrument(skip(sysinfo))]
async fn launch_services(sysinfo: SystemInfo) -> Result<()> {
    tracing::info!("Starting gRPC Server at 0.0.0.0:50051");

    tonic::transport::Server::builder()
        .concurrency_limit_per_connection(32)
        .add_service(
            events::EventsService {
                system_info: sysinfo.clone(),
            }
            .into_server(),
        )
        .add_service(hello_world::HelloWorldImpl { sysinfo }.into_server())
        .serve("0.0.0.0:50051".parse()?)
        .await
        .context("Running Tonic Unauthenticated Server")
}

#[tokio::main]
async fn main() -> Result<()> {
    application::init_tracing();

    let args = application::Args::parse();

    tracing::info!("{args:#?}");

    match args.subcommand {
        None => start_server().await,
        Some(SubCommand::Validate { creating }) => {
            validate_config(args.config_location_or_default(), creating).await
        }
    }
}

async fn start_server() -> Result<()> {
    let rx = information::start_sysinfo();

    tokio::task::Builder::new()
        .name("gRPC Server")
        .spawn(launch_services(rx.clone()))
        .await??;

    info!("Ended with value: {:#?}", rx.borrow());

    Ok(())
}

async fn validate_config(config_location: PathBuf, creating: bool) -> Result<()> {
    let config = Configuration::get(&config_location).or_else(|e| {
        if let Ok(e) = e.downcast::<std::io::Error>() {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e).map_err(Into::into);
            }
        }

        let def: Configuration = Default::default();
        def.put(&config_location).map(|_| def)
    })?;

    AuthManager::validate(
        &AuthManagerConfig {
            users_file: config.users,
        },
        creating,
    )?;

    Ok(())
}
