//! The development service provides a way to run a parachain runtime in a real blockchain context,
//! but without a backing relay chain. This allows developers to quickly and easily spin up parachain
//! nodes using the --dev flag, for example. It can also be used in integration tests.

// std
use std::time::Duration;

// Cumulus Imports
use cumulus_primitives_parachain_inherent::MockValidationDataInherentDataProvider;

// Substrate Imports
use crate::service::new_partial;
use sc_client_api::HeaderBackend;
use sc_consensus_manual_seal::consensus::aura::AuraConsensusDataProvider;
use sc_service::{Configuration, Error as ServiceError, TaskManager};

/// Builds a new development service. This service uses manual seal, and mocks
/// the parachain inherent.
pub fn new_dev(mut config: Configuration) -> Result<TaskManager, ServiceError> {
    use async_io::Timer;
    use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};
    use sp_core::H256;

    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue: _,
        keystore_container,
        select_chain: _,
        transaction_pool,
        other: (_block_import, telemetry, _telemetry_worker_handle),
    } = new_partial(&mut config)?;

    // We don't use the block import or import queue provided from new_partial
    // because they are for a parachain, and will mark new blocks as
    // not best (because parachains wait for the relay chain to do that)
    let block_import = client.clone();

    let import_queue = sc_consensus_manual_seal::import_queue(
        Box::new(block_import.clone()),
        &task_manager.spawn_essential_handle(),
        None, //TODO Re-evaluate this.
		// Using None avoids "Failed to register Prometheus metrics: Duplicate metrics collector registration attempted"
		// That might just be a warning though. Ultimately the node crashes with "Essential task `basic-block-import-worker` failed. Shutting down service."
    );

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
            Timer::interval(Duration::from_millis(12_000)),
            |_| EngineCommand::SealNewBlock {
                create_empty: true,
                finalize: false,
                parent_hash: None,
                sender: None,
            },
        ));

        let select_chain = sc_consensus::LongestChain::new(backend.clone());

        let client_set_aside_for_cidp = client.clone();

        // Create channels for mocked XCM messages.
        // let (downward_xcm_sender, downward_xcm_receiver) = flume::bounded::<Vec<u8>>(100);
        // let (hrmp_xcm_sender, hrmp_xcm_receiver) = flume::bounded::<(ParaId, Vec<u8>)>(100);
        // xcm_senders = Some((downward_xcm_sender, hrmp_xcm_sender));

        task_manager.spawn_essential_handle().spawn_blocking(
            "authorship_task",
            Some("block-authoring"),
            run_manual_seal(ManualSealParams {
                block_import,
                env,
                client: client.clone(),
                pool: transaction_pool.clone(),
                commands_stream,
                select_chain,
                consensus_data_provider: Some(Box::new(AuraConsensusDataProvider::new(
                    client.clone(),
                ))),
                create_inherent_data_providers: move |block: H256, ()| {
                    let current_para_block = client_set_aside_for_cidp
                        .number(block)
                        .expect("Header lookup should succeed")
                        .expect("Header passed in as parent should be present in backend.");

                    // let downward_xcm_receiver = downward_xcm_receiver.clone();
                    // let hrmp_xcm_receiver = hrmp_xcm_receiver.clone();

                    // let client_for_xcm = client_set_aside_for_cidp.clone();
                    async move {
                        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                        let mocked_parachain = MockValidationDataInherentDataProvider {
                            current_para_block,
                            relay_offset: 1000,
                            relay_blocks_per_para_block: 2,
                            para_blocks_per_relay_epoch: 10,
                            relay_randomness_config: (),
                            xcm_config: Default::default(),
                            raw_downward_messages: Default::default(),
                            raw_horizontal_messages: Default::default(),
                        };

                        Ok((timestamp, mocked_parachain))
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
