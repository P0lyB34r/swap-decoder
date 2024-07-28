mod metamask;
mod oneinch_v4;
mod oneinch_v5;
mod paraswap_v5;
mod uniswap_v3;
mod universal_router;
mod zerox;

use std::{cell::Cell, future::IntoFuture, io::Read, sync::Once};

use alloy::{
    eips::BlockNumberOrTag,
    primitives::{Address, TxHash, U256},
    providers::{ext::TraceApi, network::TransactionResponse, Provider, ProviderBuilder},
    rpc::types::{
        trace::{
            geth::TraceResult,
            parity::{TraceResults, TraceType},
        },
        Index, Transaction, TransactionReceipt,
    },
};
use eyre::{eyre, OptionExt};
pub use paraswap_v5::*;
pub use uniswap_v3::*;
pub use universal_router::*;

pub struct Swap {
    pub from_address: Address,
    pub to_address: Address,
    pub input_token: Address,
    pub output_token: Address,
    pub input_amount: U256,
    pub output_amount: U256,
}

pub async fn get_tx(url: &String, hash: &TxHash) -> eyre::Result<Transaction> {
    let provider = ProviderBuilder::new().on_http(url.parse()?);
    let receipt = provider
        .get_transaction_by_hash(hash.to_owned())
        .await?
        .ok_or_eyre(format!("transaction not found: {}", hash))?;
    Ok(receipt)
}

pub async fn get_tx_trace(url: &String, hash: &TxHash) -> eyre::Result<TraceResults> {
    let provider = ProviderBuilder::new().on_http(url.parse()?);
    let trace = provider
        .trace_replay_transaction(hash.to_owned(), &[TraceType::Trace])
        .await?;
    Ok(trace)
}

pub async fn get_tx_by_pos(url: &String, block: &u64, index: &u64) -> eyre::Result<Transaction> {
    let provider = ProviderBuilder::new().on_http(url.parse()?);
    let tx = provider
        .raw_request::<(BlockNumberOrTag, Index), Transaction>(
            "eth_getTransactionByBlockNumberAndIndex".into(),
            ((*block).into(), (*index as usize).into()),
        )
        .await?;
    Ok(tx)
}

pub async fn get_tx_receipt(url: &String, hash: &TxHash) -> eyre::Result<TransactionReceipt> {
    let provider = ProviderBuilder::new().on_http(url.parse()?);
    let receipt = provider
        .get_transaction_receipt(hash.to_owned())
        .await?
        .ok_or_eyre(format!("transaction not found: {}", hash))?;
    Ok(receipt)
}

pub trait Decoder {
    fn name(&self) -> String;
    fn supported_address(&self) -> Vec<Address>;
    fn supported_selectors(&self) -> Vec<[u8; 4]>;
    fn decode(&self, context: &DecoderContext) -> eyre::Result<Swap>;
}

pub enum TxPos {
    Hash(TxHash),
    Pos(u64, u64),
}

pub struct DecoderContext {
    rt: tokio::runtime::Runtime,
    rpc_url: String,

    tx: Option<Transaction>,
    receipt: Option<TransactionReceipt>,
    trace: Option<TraceResults>,
}

impl DecoderContext {
    pub fn new(rpc_url: String) -> eyre::Result<Self> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(Self {
            rt,
            rpc_url,

            tx: None,
            receipt: None,
            trace: None,
        })
    }

    pub fn decode(&mut self, pos: TxPos) -> eyre::Result<()> {
        let decoders: Vec<Box<dyn Decoder>> = vec![Box::new(DecoderUnivesalRouter::new())];
        let tx = match pos {
            TxPos::Hash(hash) => self.rt.block_on(get_tx(&self.rpc_url, &hash))?,
            TxPos::Pos(block, index) => {
                self.rt
                    .block_on(get_tx_by_pos(&self.rpc_url, &block, &index))?
            }
        };
        let (receipt, trace) = self.rt.block_on(async {
            tokio::join!(
                get_tx_receipt(&self.rpc_url, &tx.hash).into_future(),
                get_tx_trace(&self.rpc_url, &tx.hash).into_future(),
            )
        });
        self.tx.replace(tx);
        self.receipt.replace(receipt?);
        self.trace.replace(trace?);

        let tx = self.tx.as_ref().unwrap();
        let to_addr = tx
            .to()
            .ok_or(eyre!("creation transaction is not supported"))?;
        let selector = extract_selector(tx)?;
        for decoder in decoders {
            if !decoder.supported_address().contains(&to_addr) {
                continue;
            }
            if !decoder.supported_selectors().contains(&selector) {
                continue;
            }
            let swap_info = decoder.decode(&self)?;
        }
        Ok(())
    }

    pub fn tx(&self) -> &Transaction {
        self.tx.as_ref().unwrap()
    }

    pub fn trace(&self) -> &TraceResults {
        self.trace.as_ref().unwrap()
    }
}

fn extract_selector(tx: &Transaction) -> eyre::Result<[u8; 4]> {
    let mut selector = [0 as u8; 4];
    match tx.input.take(4).read(&mut selector)? {
        4 => {}
        _ => return Err(eyre!("insufficient calldata")),
    }
    Ok(selector)
}
