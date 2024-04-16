//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use jsonrpsee::RpcModule;

/// Full client dependencies.
pub struct FullDeps {
    // As you add RPC methods, you will likely need to add components to
    // fetch data from. It is common to find the client or tx pool here.
    // You will also need to add generic params and trait bounds as required.
    // See the upstream Substrate node template for more details.
}

/// Instantiate all full RPC extensions.
pub fn create_full(
    _deps: FullDeps,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>> {
    let module = RpcModule::new(());
    // Extend this RPC with a custom API by using the following syntax.
    // `YourRpcStruct` should have a reference to a client, which is needed
    // to call into the runtime.
    // `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`
    Ok(module)
}
