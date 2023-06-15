#[subxt::subxt(runtime_metadata_path = "../resources/goro.metadata")]
pub mod goro {}

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

pub const GORO: u128 = 1_000_000_000;
pub const SENDER_SIGNER: &str = "//Alice";
pub const RECEIVER_KRESNA: &str = "gr3LnU3fFLSC9CQtLQbQkZALFEZk81PES5YZV3CB8sdhQTrat";
pub const TRANSFER_AMOUNT: u128 = 123 * GORO;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Addresses
    set_default_ss58_version(Ss58AddressFormatRegistry::GoroAccount.into());
    let sender_as_keypair = Sr25519KeyPair::from_string(SENDER_SIGNER, None)?;
    let receiver_as_account = AccountId32::from_ss58check(RECEIVER_KRESNA)?;

    // Message
    let sender_as_signer = PairSigner::<PolkadotConfig, Sr25519KeyPair>::new(sender_as_keypair);
    let receiver_as_multiaddress = MultiAddress::from(receiver_as_account);
    let extrinsic_message = extrinsic().balances().transfer(receiver_as_multiaddress, TRANSFER_AMOUNT);

    // API
    let extrinsic_client = OnlineClient::<PolkadotConfig>::from_url("wss://main-00.goro.network:443").await?;
    let extrinsic_progress = extrinsic_client
        .tx()
        .sign_and_submit_then_watch_default(&extrinsic_message, &sender_as_signer)
        .await?;
    println!("\n[Transfer Sent]");
    println!("=> hash     : {}", extrinsic_progress.extrinsic_hash());

    // Event filtering
    let extrinsic_events = extrinsic_progress.wait_for_in_block().await?.wait_for_success().await?;

    if let Some(transfer_event) = extrinsic_events.find_first::<goro::balances::events::Transfer>()? {
        let goro::balances::events::Transfer { from, to, amount } = transfer_event;

        println!("\n[Transfer In-Block]");
        println!("=> from     : {}", AccountId32::new(from.0));
        println!("=> to       : {}", AccountId32::new(to.0));
        println!("=> amount   : {}", (amount as f32) / (GORO as f32));
    }

    Ok(())
}
