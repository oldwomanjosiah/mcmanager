const PROTOS: &[&str] = &[
    "proto/helloworld/helloworld.proto",
    "proto/event/event.proto",
];
const INCLUDES: &[&str] = &["proto"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .compile(PROTOS, INCLUDES)?;

    Ok(())
}
