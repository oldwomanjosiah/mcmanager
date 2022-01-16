const PROTOS: &'static [&'static str] = &[
    "proto/helloworld/helloworld.proto",
    "proto/event/event.proto",
];
const INCLUDES: &'static [&'static str] = &["proto"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .compile(PROTOS, INCLUDES)?;

    Ok(())
}
