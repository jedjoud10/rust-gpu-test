use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("crates/raymarch", "spirv-unknown-vulkan1.1")
        .capability(Capability::VariablePointers)
        .build()?;
    SpirvBuilder::new("crates/blit", "spirv-unknown-vulkan1.1")
        .capability(Capability::VariablePointers)
        .build()?;
    Ok(())
}