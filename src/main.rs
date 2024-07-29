#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

use alloy::{
    primitives::b256,
    providers::{Provider, ProviderBuilder},
};
use interfaces::get_tx_by_pos;

use polars::{lazy::prelude::*, prelude::*};

mod interfaces;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio");
    let handle = rt.handle();

    let lazy =
        LazyFrame::scan_parquet("./temp/data/all", ScanArgsParquet::default()).expect("lazy");
        
    let tx = interfaces::TxPos::Hash(b256!(
        "2852362de2c7c05050d7b8c10945aa6161f7bbdc34f136068c28205f32a8308a"
    ));
    let rpc = "http://192.168.0.105:8545".to_string();
    interfaces::DecoderContext::decode(rpc, tx).expect("decode");
}
