#[subxt::subxt(runtime_metadata_path = "../resources/goro.metadata")]
pub mod goro {}

use crate::goro::runtime_types::sp_weights::weight_v2::Weight;
use contract_transcode::ContractMessageTranscoder;
use goro::tx as extrinsic;
use ss58_registry::Ss58AddressFormatRegistry;
use subxt::ext::sp_core::crypto::set_default_ss58_version;
use subxt::ext::sp_core::sr25519::Pair as Sr25519KeyPair;
use subxt::ext::sp_core::Pair;
use subxt::ext::sp_runtime::app_crypto::Ss58Codec;
use subxt::ext::sp_runtime::AccountId32;
use subxt::tx::PairSigner;
use subxt::utils::MultiAddress;
use subxt::{OnlineClient, PolkadotConfig};

pub const CALLER_SIGNER: &str = "//Alice";
pub const CONTRACT_ADDRESS: &str = "gr6baaPGVaHSgHCy98gmHb7nfaDLPNSLfHApvJ8wC2TEbSKGP";
pub const CONTRACT_METADATA: &str = "resources/goro_flipper_demo.json";
pub const CONTRACT_METHOD_SET: &str = "set";
pub const CONTRACT_METHOD_GET: &str = "get";
pub const LIMIT_PROOF_SIZE: u64 = 131_072;
pub const LIMIT_REF_TIME: u64 = 8_589_934_592;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Addresses
    set_default_ss58_version(Ss58AddressFormatRegistry::GoroAccount.into());
    let sender_as_keypair = Sr25519KeyPair::from_string(CALLER_SIGNER, None)?;
    let contract_as_account = AccountId32::from_ss58check(CONTRACT_ADDRESS)?;

    // Message
    let sender_as_signer = PairSigner::<PolkadotConfig, Sr25519KeyPair>::new(sender_as_keypair);
    let gas_limit = Weight {
        proof_size: LIMIT_PROOF_SIZE,
        ref_time: LIMIT_REF_TIME,
    };
    let contract_transcoder = ContractMessageTranscoder::load(CONTRACT_METADATA)?;
    let write_message = contract_transcoder.encode(CONTRACT_METHOD_SET, ["true"])?;
    let contract_as_multiaddress = MultiAddress::from(contract_as_account);
    let extrinsic_message = extrinsic()
        .contracts()
        .call(contract_as_multiaddress, 0, gas_limit, None, write_message);

    // API
    let extrinsic_client = OnlineClient::<PolkadotConfig>::from_url("wss://main-00.goro.network:443").await?;
    let extrinsic_progress = extrinsic_client
        .tx()
        .sign_and_submit_then_watch_default(&extrinsic_message, &sender_as_signer)
        .await?;
    println!("\n[Contract Call Sent]");
    println!("=> hash     : {}", extrinsic_progress.extrinsic_hash());

    // Event filtering
    let extrinsic_events = extrinsic_progress.wait_for_in_block().await?.wait_for_success().await?;

    if let Some(contract_event) = extrinsic_events.find_first::<goro::contracts::events::Called>()? {
        let goro::contracts::events::Called { caller, contract } = contract_event;

        println!("\n[Contract Called]");
        println!("=> caller   : {}", AccountId32::new(caller.0));
        println!("=> contract : {}", AccountId32::new(contract.0));
    }

    Ok(())
}
