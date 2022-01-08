extern crate anyhow;
extern crate clap;
extern crate console_subscriber;
extern crate tokio;
extern crate tracing;
extern crate tracing_subscriber;

use clap::StructOpt;
use information::SystemInfo;
use tracing::info;

mod application;
mod information;
mod prelude;
mod util;

use prelude::*;

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

#[tracing::instrument]
async fn launch_services(sysinfo: SystemInfo) -> Result<()> {
    tracing::info!("Starting gRPC Server at 0.0.0.0:50051");
    tonic::transport::Server::builder()
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

    tracing::info!("{:#?}", args);

    let rx = information::start_sysinfo();

    tokio::task::Builder::new()
        .name("gRPC Server")
        .spawn(launch_services(rx.clone()))
        .await??;

    info!("Ended with value: {:#?}", rx.borrow());

    Ok(())
}
