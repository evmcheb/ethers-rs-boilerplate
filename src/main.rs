mod bindings;
mod trace;
use anyhow::Result;
use std::{env, str::FromStr, sync::Arc};

use bindings::erc_20::erc20;
use dotenv::dotenv;
use ethers::{
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer},
    types::{Address, Eip1559TransactionRequest, Filter, U256},
    utils::parse_units,
};
use futures::StreamExt;

// Example block subscription filter
async fn block_loop(client: Arc<Provider<Ws>>) -> Result<()> {
    let mut stream = client.subscribe_blocks().await?;
    while let Some(block) = stream.next().await {
        // Print the block hash
        println!("New block: {:?} {}", block.hash, block.timestamp);
        // Get a debug trace of the first transaction
        if let Some(tx) = block.transactions.first() {
            let full_tx = client.get_transaction(*tx).await?;
            if let Some(tx) = full_tx {
                let flattened = trace::get_flattened_trace(tx.clone(), client.clone()).await;
                println!("Flattened trace: {:?}", flattened);
            }
        }
    }
    Ok(())
}

// Example event subscription filter
async fn filter_loop(client: Arc<Provider<Ws>>, f: Filter) -> Result<()> {
    let mut stream = client.subscribe_logs(&f).await?;
    while let Some(log) = stream.next().await {
        // Maybe decode some data here
        let amount = U256::from_big_endian(&log.data[0..32]);
        println!("New transfer: {:?} {:?}", log, amount);
    }
    Ok(())
}

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set in .env");
    let rpc = env::var("RPC").expect("RPC must be set in .env");

    let client = Arc::new(Provider::<Ws>::connect(rpc.clone()).await?);
    let chain_id = client.get_chainid().await?.as_u64();

    // A transaction signer using the .env private key
    let signer: LocalWallet = private_key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(chain_id);

    // Initializing an example contract
    let weth = Arc::new(erc20::new(
        Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap(),
        client.clone(),
    ));

    // Example eth_sendRawTransaction
    let nonce = client.get_transaction_count(signer.address(), None).await?;
    let data = weth
        .transfer(signer.address(), U256::from(100))
        .calldata()
        .ok_or("Failed to encode calldata")?;
    let tx = Eip1559TransactionRequest::new()
        .to(weth.address())
        .value(U256::zero())
        .nonce(nonce)
        .max_fee_per_gas(parse_units("1", "gwei").unwrap())
        .max_priority_fee_per_gas(parse_units("0.1", "gwei").unwrap())
        .gas(U256::from(2_000_000))
        .data(data)
        .into();

    let signed_tx = signer.sign_transaction(&tx).await?;
    let raw = tx.rlp_signed(&signed_tx);
    let receipt = client.send_raw_transaction(raw).await?.await?;
    match receipt {
        Some(receipt) => {
            println!("Tx landed: {:?}", receipt.transaction_hash);
        }
        None => {
            println!("Tx didn't land");
        }
    }

    let transfer_filter = weth.transfer_filter().filter;
    let block_handle = tokio::spawn(block_loop(client.clone()));
    let filter_handle = tokio::spawn(filter_loop(client.clone(), transfer_filter));

    // Loop forever
    loop {
        tokio::select! {
            r = block_handle => {
                println!("Block loop exited with {:?}", r);
                break;
            }
            r = filter_handle => {
                println!("Filter loop exited with {:?}", r);
                break;
            }
        }
    }

    Ok(())
}
