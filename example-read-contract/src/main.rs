pub(crate) mod logger;

use contract_transcode::{ContractMessageTranscoder};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::WsClientBuilder;
use parity_scale_codec::{Encode, Decode};
use sp_core::Bytes;
use sp_core::crypto::{set_default_ss58_version, AccountId32, Ss58Codec};
use ss58_registry::Ss58AddressFormatRegistry;
use std::time::Duration;
use pallet_contracts_primitives::{ContractExecResult};

pub const CONTRACT_CALLER: &str = "gr5wupneKLGRBrA3hkcrXgbwXp1F26SV7L4LymGxCKs9QMXn1";
pub const CONTRACT_ADDRESS: &str = "gr6baaPGVaHSgHCy98gmHb7nfaDLPNSLfHApvJ8wC2TEbSKGP";
pub const CONTRACT_METADATA: &str = "../resources/goro_flipper_demo.json";
pub const CONTRACT_METHOD_GET: &str = "get";

#[derive(Encode)]
pub struct Weight {
    proof_size: u64,
    ref_time: u64,
}
#[derive(Encode)]
pub struct ContractCallEnvelope {
    origin: AccountId32,
    dest: AccountId32,
    value: u128,
    gas_limit: Option<Weight>,
    storage_deposit_limit: Option<u128>,
    input_data: Vec<u8>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_logger();

    set_default_ss58_version(Ss58AddressFormatRegistry::GoroAccount.into());
    let contract_transcoder = ContractMessageTranscoder::load(CONTRACT_METADATA)?;
    let contract_caller = AccountId32::from_ss58check(CONTRACT_CALLER)?;
    let contract_address = AccountId32::from_ss58check(CONTRACT_ADDRESS)?;
    let contract_message_data = contract_transcoder.encode(CONTRACT_METHOD_GET, [""; 0])?;
    let contract_call_envelope = ContractCallEnvelope {
        origin: contract_caller,
        dest: contract_address,
        value: 0,
        gas_limit: None,
        storage_deposit_limit: None,
        input_data: contract_message_data,
    };
    let contract_call_envelope_bytes = contract_call_envelope.encode();
    let contract_call_envelope_as_rpc_params = rpc_params!["ContractsApi_call", Bytes(contract_call_envelope_bytes)];
    let goro_client = WsClientBuilder::default()
        .max_concurrent_requests(1)
        .ping_interval(Duration::from_secs(1))
        .request_timeout(Duration::from_secs(5))
        .build("wss://main-00.goro.network:443")
        .await?;

    let result: Bytes = goro_client.request("state_call", contract_call_envelope_as_rpc_params).await?;
    let contract_exec_result = ContractExecResult::<u128>::decode(&mut result.as_ref())?;

    match contract_exec_result.result {
        Err(err) => {
            logger::error!("{err:?}");
        }
        Ok(ref exec_return_value) => {
            let value = contract_transcoder.decode_return(CONTRACT_METHOD_GET, &mut &exec_return_value.data[..])?;

            logger::info!("Result => {}", value);
        }
    }

    Ok(())
}
