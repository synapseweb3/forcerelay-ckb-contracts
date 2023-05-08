mod client_type_lock;
mod verify_bin;

mod utils;

const DATA_DIR: &str = "data/eth_light_client";
const CLIENT_TYPE_LOCK_CONTRACT: &str = "../build/eth_light_client-client_type_lock";
const VERIFY_BIN_CONTRACT: &str = "../build/eth_light_client-verify_bin";
const MOCK_BUSINESS_TYPE_LOCK_CONTRACT: &str = "../build/eth_light_client-mock_business_type_lock";
