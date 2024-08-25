use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("crates/shader", "spirv-unknown-vulkan1.1")
        .capability(Capability::VariablePointers)
        .build()?;
    Ok(())
}