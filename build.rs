use std::{fs::read_dir, path::Path};
use spirv_builder::{Capability, SpirvBuilder};

/*
fn rerun_directory<T: AsRef<Path> + ?Sized>(dir: &T) {
    println!("cargo:rerun-if-changed={}", dir.as_ref().to_str().unwrap());
    // Find any other directories in this one.
    for entry in read_dir(dir).unwrap() {
        let entry = entry.expect("Couldn't access file in src directory");
        let path = entry.path().to_path_buf();
        // Skip this entry if it isn't a directory.
        if !path.is_dir() {
            continue;
        }
        rerun_directory(&path);
    }
}
*/

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=crates");
    SpirvBuilder::new("crates/shader", "spirv-unknown-vulkan1.2")
        .print_metadata(spirv_builder::MetadataPrintout::None)
        //.extension("SPV_EXT_debug_info")
        .capability(Capability::VariablePointers)
        .capability(Capability::StorageImageArrayNonUniformIndexing)
        .capability(Capability::StorageImageArrayDynamicIndexing)
        .capability(Capability::StorageImageExtendedFormats)
        //.capability(Capability::VulkanMemoryModelDeviceScope)
        .multimodule(true)
        .release(false)
        .build().unwrap()
        .module
        .unwrap_multi()
        .iter()
        .for_each(|(entry, path)| {
            println!("cargo:rustc-env={entry}={}", path.to_str().unwrap());
            println!("cargo:warning={}", format!("{entry}={}", path.to_str().unwrap()))
        });

    Ok(())
}
