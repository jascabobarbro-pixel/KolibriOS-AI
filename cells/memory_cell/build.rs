fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &[
                "../protos/cell_common.proto",
                "../protos/memory_cell.proto",
            ],
            &["../protos/"],
        )?;
    Ok(())
}
