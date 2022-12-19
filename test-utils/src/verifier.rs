use tempfile::TempDir;

use ckb_db::RocksDB;
use ckb_db_schema::COLUMNS;
use ckb_error::Error;
use ckb_script::TransactionScriptsVerifier;
use ckb_store::{data_loader_wrapper::DataLoaderWrapper, ChainDB};
use ckb_types::core::{cell::ResolvedTransaction, Cycle};

pub struct Verifier {
    store: ChainDB,
    _tmp_dir: TempDir,
}

impl Default for Verifier {
    fn default() -> Self {
        let tmp_dir = TempDir::new().unwrap();
        let db = RocksDB::open_in(&tmp_dir, COLUMNS);
        let store = ChainDB::new(db, Default::default());
        Self {
            store,
            _tmp_dir: tmp_dir,
        }
    }
}

impl Verifier {
    pub fn verify_without_limit(&self, rtx: &ResolvedTransaction) -> Result<Cycle, Error> {
        self.verify(rtx, u64::MAX)
    }

    pub fn verify(&self, rtx: &ResolvedTransaction, max_cycles: Cycle) -> Result<Cycle, Error> {
        self.verify_map(rtx, |verifier| verifier.verify(max_cycles))
    }

    pub fn verify_map<R, F>(&self, rtx: &ResolvedTransaction, mut verify_func: F) -> R
    where
        F: FnMut(TransactionScriptsVerifier<'_, DataLoaderWrapper<'_, ChainDB>>) -> R,
    {
        let data_loader = DataLoaderWrapper::new(&self.store);
        let verifier = TransactionScriptsVerifier::new(rtx, &data_loader);
        verify_func(verifier)
    }
}
