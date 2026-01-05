fn main() -> Result<(), Box<dyn std::error::Error>> {
	let proto_file = "../../proto/sluice/v1/sluice.proto";

	tonic_build::configure()
		.build_client(true)
		.build_server(false)
		.compile(&[proto_file], &["../../proto"])?;

	println!("cargo:rerun-if-changed={proto_file}");
	println!("cargo:rerun-if-changed=../../proto/sluice/v1");

	Ok(())
}
