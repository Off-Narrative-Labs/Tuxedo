#[cfg(feature = "std")]
fn main() {
    substrate_wasm_builder::WasmBuilder::new()
        .with_current_project()
        .disable_feature("parachain-std")
        // Not sure whether I need to explicitly enable this one or not.
        // I think not, but better safe than sorry at first.
        .enable_feature("parachain")
        .export_heap_base()
        .import_memory()
        .build()
}

/// The wasm builder is deactivated when compiling
/// this crate for wasm to speed up the compilation.
#[cfg(not(feature = "std"))]
fn main() {}
