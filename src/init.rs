use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::{
    externals::TreeExternals, BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree,
};
use reth_db::mdbx::{Env, EnvKind, NoWriteMap};
use reth_network_api::test_utils::NoopNetwork;
use reth_primitives::MAINNET;
use reth_provider::{providers::BlockchainProvider, ShareableDatabase};
use reth_revm::Factory;
use reth_rpc::{
    eth::{
        cache::{EthStateCache, EthStateCacheConfig},
        gas_oracle::{GasPriceOracle, GasPriceOracleConfig},
    },
    EthApi, EthFilter,
};
use reth_transaction_pool::EthTransactionValidator;
use std::{path::Path, sync::Arc};

use crate::{RethApi, RethClient, RethFilter, RethTrace, RethTxPool};

// EthApi/Filter Client
pub fn init_client(db_path: &Path) -> RethClient {
    let chain = Arc::new(MAINNET.clone());
    let db = Arc::new(Env::<NoWriteMap>::open(db_path.as_ref(), EnvKind::RO).unwrap());

    let tree_externals = TreeExternals::new(
        Arc::clone(&db),
        Arc::new(BeaconConsensus::new(Arc::clone(&chain))),
        Factory::new(Arc::clone(&chain)),
        Arc::clone(&chain),
    );

    let tree_config = BlockchainTreeConfig::default();
    let (canon_state_notification_sender, _receiver) =
        tokio::sync::broadcast::channel(tree_config.max_reorg_depth() as usize * 2);

    let blockchain_tree = ShareableBlockchainTree::new(
        BlockchainTree::new(tree_externals, canon_state_notification_sender.clone(), tree_config)
            .unwrap(),
    );

    let blockchain_db = BlockchainProvider::new(
        ShareableDatabase::new(Arc::clone(&db), Arc::clone(&chain)),
        blockchain_tree,
    )
    .unwrap();

    blockchain_db
}

/// EthApi
pub fn init_eth_api(client: RethClient) -> RethApi {
    let tx_pool = init_pool(client.clone());

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let api = EthApi::new(
        client.clone(),
        tx_pool,
        NoopNetwork,
        state_cache,
        GasPriceOracle::new(client.clone(), GasPriceOracleConfig::default(), state_cache.clone()),
    );

    api
}

/// EthFilter
pub fn init_eth_filter(client: RethClient, max_logs_per_response: usize) -> RethFilter {
    let tx_pool = init_pool(client.clone());

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let filter = EthFilter::new(client, tx_pool, state_cache, max_logs_per_response);

    filter
}

// EthApi/Filter txPool
pub fn init_pool(client: RethClient) -> RethTxPool {
    let chain = Arc::new(MAINNET.clone());

    let tx_pool = reth_transaction_pool::Pool::eth_pool(
        EthTransactionValidator::new(client, chain),
        Default::default(),
    );

    tx_pool
}

// RethTrace
pub fn init_Trace() -> RethTrace {
    todo!()
}