use alloy::{
    primitives::Address, providers::network::TransactionResponse, rpc::types::Transaction, sol, sol_types::{SolCall, SolInterface}
};
use eyre::eyre;

use super::Decoder;

mod consts {
    use alloy::primitives::{address, b256, Address, B256};

    pub const NAME: &str = "MetaMask Swap Router";
    pub const ROUTER: Address = address!("881d40237659c251811cec9c364ef91dc08d300c");
}

sol! {

// cast interface 0x881d40237659c251811cec9c364ef91dc08d300c
interface MetaSwap {
    event AdapterRemoved(string indexed aggregatorId);
    event AdapterSet(string indexed aggregatorId, address indexed addr, bytes4 selector, bytes data);
    event Swap(string indexed aggregatorId, address indexed sender);

    function swap(string memory aggregatorId, address tokenFrom, uint256 amount, bytes memory data) external payable;
    function swapUsingGasToken(string memory aggregatorId, address tokenFrom, uint256 amount, bytes memory data)
        external
        payable;
}

}

// Spec:
// swap_router = 0x881d40237659c251811cec9c364ef91dc08d300c
// airswap_spender =
// swap_router.swap -> airswap_spender.swap -> adapter
// selector = predefined

// {'openOceanFeeDynamic': 44488,
//              'oneInchV5FeeDynamic': 785774,
//              '0xFeeDynamic': 61058,
//              'airswapLight4FeeDynamicFixed': 92945,
//              'paraswapV5FeeDynamic': 17404,
//              'pmmFeeDynamicv4': 115742,
//              'hashFlowFeeDynamic': 447,
//              'airswapLight3FeeDynamic': 12,
//              'oneInchV4FeeDynamic': 41,
//              'oneInchV3FeeDynamic': 13,
//              'kyberSwapFeeDynamic': 12781,
//              'bebopMultiFeeDynamic': 7,
//              'oneInchV3': 3,
//              'paraswapV4': 1}

pub struct DecoderMetaMaskSwapRouter {}

impl DecoderMetaMaskSwapRouter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Decoder for DecoderMetaMaskSwapRouter {
    fn name(&self) -> String {
        consts::NAME.to_string()
    }

    fn supported_address(&self) -> Vec<Address> {
        vec![consts::ROUTER]
    }

    fn supported_selectors(&self) -> Vec<[u8; 4]> {
        vec![MetaSwap::swapCall::SELECTOR]
    }

    fn decode(&self, context: &super::DecoderContext) -> eyre::Result<super::Swap> {
        let tx = context.tx();

        match MetaSwap::MetaSwapCalls::abi_decode(&tx.input, true)? {
            MetaSwap::MetaSwapCalls::swap(call) => {
                let input_token = call.tokenFrom;
                let input_amount = call.amount;
                let from_address = tx.from();
                let to_address = tx.from();

                match call.aggregatorId.as_str() {
                    "oneInchV5FeeDynamic" => {}
                    _ => return Err(eyre!("MetaMask Swap Router: unspported aggregator: ")),
                }
            }
            _ => return Err(eyre!("MetaMask Swap Router: unsupported selector: ")),
        }
        todo!()
    }
}
