extern crate clap;
extern crate console_subscriber;
extern crate tokio;
extern crate tracing;
extern crate tracing_subscriber;

use clap::StructOpt;

mod application;

mod hello_world {
    tonic::include_proto!("helloworld");

    pub struct HelloWorldServiceImpl;

    #[tonic::async_trait]
    impl hello_world_service_server::HelloWorldService for HelloWorldServiceImpl {
        async fn hello_world(
            &self,
            request: tonic::Request<HelloRequest>,
        ) -> Result<tonic::Response<HelloResponse>, tonic::Status> {
            Ok(tonic::Response::new(HelloResponse {
                greeting: format!("Hello, {}!", request.into_inner().name),
            }))
        }
    }
}

#[tracing::instrument]
async fn launch_services() -> Result<(), tonic::transport::Error> {
    tonic::transport::Server::builder()
        .add_service(
            hello_world::hello_world_service_server::HelloWorldServiceServer::new(
                hello_world::HelloWorldServiceImpl,
            ),
        )
        .serve("0.0.0.0:50051".parse().unwrap())
        .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    application::init_tracing();

    let args = application::Args::parse();

    tracing::info!("{:#?}", args);

    tokio::task::Builder::new()
        .name("gRPC Server")
        .spawn(launch_services())
        .await??;

    Ok(())
}
