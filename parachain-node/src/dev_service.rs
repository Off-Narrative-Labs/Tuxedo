//! The development service provides a way to run a parachain runtime in a real blockchain context,
//! but without a backing relay chain. This allows developers to quickly and easily spin up parachain
//! nodes using the --dev flag, for example. It can also be used in integration tests.

use parachain_template_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::BlockBackend;
use sc_consensus_manual_seal::consensus::aura::AuraConsensusDataProvider;
pub use sc_executor::NativeElseWasmExecutor;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use std::{sync::Arc, time::Duration};
use tuxedo_core::genesis::TuxedoGenesisBlockBuilder;

use cumulus_primitives_parachain_inherent::MockValidationDataInherentDataProvider;

// Our native executor instance.
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        parachain_template_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        parachain_template_runtime::native_version()
    }
}

pub(crate) type FullClient =
    sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

#[allow(clippy::type_complexity)]
pub fn new_partial(
    config: &Configuration,
) -> Result<
    sc_service::PartialComponents<
        FullClient,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block>,
        sc_transaction_pool::FullPool<Block, FullClient>,
        Option<Telemetry>,
    >,
    ServiceError,
> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = sc_service::new_native_or_wasm_executor(config);

    let backend = sc_service::new_db_backend(config.db_config())?;
    let genesis_block_builder = TuxedoGenesisBlockBuilder::new(
        config.chain_spec.as_storage_builder(),
        !config.no_genesis(),
        backend.clone(),
        executor.clone(),
    )?;

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts_with_genesis_builder::<Block, RuntimeApi, _, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
            backend,
            genesis_block_builder,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let import_queue = sc_consensus_manual_seal::import_queue(
        Box::new(client.clone()),
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    );

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (telemetry),
    })
}

/// Builds a new development service. This service uses manual seal, and mocks
/// the parachain inherent.
pub fn new_dev(config: Configuration) -> Result<TaskManager, ServiceError> {
    use async_io::Timer;
    use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};

    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: telemetry,
    } = new_partial(&config)?;

    let net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params: None,
            block_relay: None,
        })?;

    let prometheus_registry = config.prometheus_registry().cloned();
    let collator = config.role.is_authority();

    if collator {
        let env = sc_basic_authorship::ProposerFactory::with_proof_recording(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let commands_stream = Box::new(futures::StreamExt::map(
            Timer::interval(Duration::from_millis(6_000)),
            |_| EngineCommand::SealNewBlock {
                create_empty: true,
                finalize: false,
                parent_hash: None,
                sender: None,
            },
        ));

        let client_for_cidp = client.clone();

        // Create channels for mocked XCM messages.
        // let (downward_xcm_sender, downward_xcm_receiver) = flume::bounded::<Vec<u8>>(100);
        // let (hrmp_xcm_sender, hrmp_xcm_receiver) = flume::bounded::<(ParaId, Vec<u8>)>(100);
        // xcm_senders = Some((downward_xcm_sender, hrmp_xcm_sender));

        task_manager.spawn_essential_handle().spawn_blocking(
            "authorship_task",
            Some("block-authoring"),
            run_manual_seal(ManualSealParams {
                block_import: client.clone(),
                env,
                client: client.clone(),
                pool: transaction_pool.clone(),
                commands_stream,
                select_chain,
                consensus_data_provider: Some(Box::new(AuraConsensusDataProvider::new(
                    client.clone(),
                ))),
                create_inherent_data_providers: move |parent_hash, ()| {
                    let maybe_parent_block = client_for_cidp.clone().block(parent_hash);

                    async move {
                        let parent_block = maybe_parent_block?
                            .ok_or(sp_blockchain::Error::UnknownBlock(parent_hash.to_string()))?
                            .block;

                        let mocked_parachain = {
                            use sp_api::{BlockT, HeaderT};

                            MockValidationDataInherentDataProvider {
                                current_para_block: parent_block.header().number() + 1,
                                relay_offset: 1000,
                                relay_blocks_per_para_block: 2,
                                para_blocks_per_relay_epoch: 10,
                                relay_randomness_config: (),
                                xcm_config: Default::default(),
                                raw_downward_messages: Default::default(),
                                raw_horizontal_messages: Default::default(),
                            }
                        };

                        let parent_idp =
                            tuxedo_core::inherents::ParentBlockInherentDataProvider(parent_block);
                        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                        Ok((parent_idp, timestamp, mocked_parachain))
                    }
                },
            }),
        );
    }

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
            };

            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network,
        client,
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool,
        rpc_builder,
        backend,
        system_rpc_tx,
        sync_service: sync_service.clone(),
        config,
        tx_handler_controller,
        telemetry: None,
    })?;

    log::info!("Development Service Ready");

    network_starter.start_network();
    Ok(task_manager)
}
