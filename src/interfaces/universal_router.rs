use std::io::Read;

use alloy::{
    primitives::{address, b256, bytes::Buf, keccak256, Address, Bytes, Signed, B256, I256, U256},
    providers::network::TransactionResponse,
    rpc::types::{
        trace::parity::{Action, CallType, TraceOutput, TransactionTrace},
        Transaction,
    },
    sol,
    sol_types::{SolCall, SolInterface, SolType, SolValue},
};
use eyre::eyre;
use serde::{Deserialize, Serialize};

use super::Decoder;

mod consts {
    use alloy::primitives::{address, b256, Address, B256};

    pub const NAME: String = "Uniswap Universal Router".to_string();
    pub const ROUTER: Address = address!("3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD");
    pub const ROUTER_V2: Address = address!("Ef1c6E67703c7BD7107eed8303Fbe6EC2554BF6B");

    pub const MSG_SENDER: Address = Address::with_last_byte(1);
    pub const ADDRESS_THIS: Address = Address::with_last_byte(2);
    pub const FEE_COLLECTOR: Address = address!("000000fee13a103a10d593b9ae06b3e05f2e7e1c");

    pub const V3_POOL_INIT_CODE_HASH: B256 =
        b256!("e34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54");
    pub const V3_FACTORY_ADDRESS: Address = address!("1F98431c8aD98523631AE4a59f267346ea31F984");

    pub const V2_PAIR_INIT_CODE_HASH: B256 =
        b256!("96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f");
    pub const V2_FACTORY_ADDRESS: Address = address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
}

sol! {

interface UniversalRouter {
    function execute(bytes memory commands, bytes[] memory inputs) external payable;
    function execute(bytes memory commands, bytes[] memory inputs, uint256 deadline) external payable;
}

interface Dispatcher {
    struct AllowanceTransferDetails {
        address from;
        address to;
        uint160 amount;
        address token;
    }

    struct PermitDetails {
        address token;
        uint160 amount;
        uint48 expiration;
        uint48 nonce;
    }

    struct PermitSingle {
        PermitDetails details;
        address spender;
        uint256 sigDeadline;
    }

    struct PermitBatch {
        PermitDetails[] details;
        address spender;
        uint256 sigDeadline;
    }

    function swap(
        address recipient,
        bool zeroForOne,
        int256 amountSpecified,
        uint160 sqrtPriceLimitX96,
        bytes calldata data
    ) external returns (int256 amount0, int256 amount1);

    function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
}

}

mod command_types {
    // Command Types where value<0x08, executed in the first nested-if block
    pub const V3_SWAP_EXACT_IN: u8 = 0x00;
    pub const V3_SWAP_EXACT_OUT: u8 = 0x01;
    pub const PERMIT2_TRANSFER_FROM: u8 = 0x02;
    pub const PERMIT2_PERMIT_BATCH: u8 = 0x03;
    pub const SWEEP: u8 = 0x04;
    pub const TRANSFER: u8 = 0x05;
    pub const PAY_PORTION: u8 = 0x06;
    pub const COMMAND_PLACEHOLDER_0x07: u8 = 0x07;

    // Command Types where 0x08<=value<=0x0f, executed in the second nested-if block
    pub const V2_SWAP_EXACT_IN: u8 = 0x08;
    pub const V2_SWAP_EXACT_OUT: u8 = 0x09;
    pub const PERMIT2_PERMIT: u8 = 0x0a;
    pub const WRAP_ETH: u8 = 0x0b;
    pub const UNWRAP_WETH: u8 = 0x0c;
    pub const PERMIT2_TRANSFER_FROM_BATCH: u8 = 0x0d;
    pub const COMMAND_PLACEHOLDER_0x0e: u8 = 0x0e;
    pub const COMMAND_PLACEHOLDER_0x0f: u8 = 0x0f;

    // Command Types where 0x10<=value<0x18, executed in the third nested-if block
    pub const SEAPORT: u8 = 0x10;
    pub const LOOKS_RARE_721: u8 = 0x11;
    pub const NFTX: u8 = 0x12;
    pub const CRYPTOPUNKS: u8 = 0x13;
    pub const LOOKS_RARE_1155: u8 = 0x14;
    pub const OWNER_CHECK_721: u8 = 0x15;
    pub const OWNER_CHECK_1155: u8 = 0x16;
    pub const SWEEP_ERC721: u8 = 0x17;

    // Command Types where 0x18<=value<=0x1f, executed in the final nested-if block
    pub const X2Y2_721: u8 = 0x18;
    pub const SUDOSWAP: u8 = 0x19;
    pub const NFT20: u8 = 0x1a;
    pub const X2Y2_1155: u8 = 0x1b;
    pub const FOUNDATION: u8 = 0x1c;
    pub const SWEEP_ERC1155: u8 = 0x1d;
    pub const COMMAND_PLACEHOLDER_0x1e: u8 = 0x1e;
    pub const COMMAND_PLACEHOLDER_0x1f: u8 = 0x1f;
}

pub struct DecoderUnivesalRouter {}

impl DecoderUnivesalRouter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Decoder for DecoderUnivesalRouter {
    fn name(&self) -> String {
        consts::NAME
    }

    fn supported_address(&self) -> Vec<Address> {
        vec![consts::ROUTER, consts::ROUTER_V2]
    }

    fn supported_selectors(&self) -> Vec<[u8; 4]> {
        use UniversalRouter as C;
        vec![C::execute_0Call::SELECTOR, C::execute_1Call::SELECTOR]
    }

    fn decode(&self, context: &super::DecoderContext) -> eyre::Result<super::Swap> {
        use UniversalRouter::UniversalRouterCalls as C;
        let msg_sender = context.tx().from();

        let (commands, inputs) = match C::abi_decode(&context.tx().input, true)? {
            C::execute_0(call) => (call.commands, call.inputs),
            C::execute_1(call) => (call.commands, call.inputs),
        };
        for (index, command) in commands.into_iter().enumerate() {
            match command {
                // Swap operations
                // For swap operations, payer is either msg.sender or address(this)
                command_types::V3_SWAP_EXACT_IN => {
                    type Params = sol!((address, uint256, uint256, bytes, bool));
                    let (recipient, amount_in, _amount_out_min, path, _payer_is_user) =
                        Params::abi_decode_params(&inputs[index], true)?;

                    let swap = v3_decode_swap(
                        SwapType::ExactIn(amount_in),
                        &path,
                        &context.trace().trace,
                    )?;
                    println!("V3_SWAP_EXACT_IN: {:?}", swap);
                }
                command_types::V3_SWAP_EXACT_OUT => {
                    type Params = sol!((address, uint256, uint256, bytes, bool));
                    let (recipient, amount_out, _amount_in_max, path, _payer_is_user) =
                        Params::abi_decode_params(&inputs[index], true)?;

                    let swap = v3_decode_swap(
                        SwapType::ExactOut(amount_out),
                        &path,
                        &context.trace().trace,
                    )?;
                    println!("V3_SWAP_EXACT_OUT: {:?}", swap);
                }
                command_types::V2_SWAP_EXACT_IN => {
                    type Params = sol!((address, uint256, uint256, address[], bool));
                    let (recipient, amount_in, amount_out_min, path, payer_is_user) =
                        Params::abi_decode_params(&inputs[index], true)?;
                    println!("V2_SWAP_EXACT_IN: {:?} {:?}", path, recipient);
                    // let swap = v2_decode_swap(&path, &context.trace().trace)?;
                }
                command_types::V2_SWAP_EXACT_OUT => {
                    type Params = sol!((address, uint256, uint256, address[], bool));
                    let (recipient, amount_out, amount_in_max, path, payer_is_user) =
                        Params::abi_decode_params(&inputs[index], true)?;

                    let swap = v2_decode_swap(&path, &context.trace().trace)?;
                }

                // non-swap commands: 0x00 <= command < 0x08
                command_types::PERMIT2_TRANSFER_FROM => {}
                command_types::PERMIT2_PERMIT_BATCH => {}
                command_types::SWEEP => {
                    type Params = sol!((address, address, uint256));
                    let (token, recipient, amount_min) =
                        Params::abi_decode_params(&inputs[index], true)?;
                    if recipient != consts::MSG_SENDER {
                        println!("SWEEP: {:?} {:?}", recipient, token);
                    }
                }
                command_types::TRANSFER => {
                    type Params = sol!((address, address, uint256));
                    let (token, recipient, value) =
                        Params::abi_decode_params(&inputs[index], true)?;
                    if recipient != consts::MSG_SENDER && recipient != consts::FEE_COLLECTOR {
                        println!("TRANSFER: {:?} {:?} {:?}", token, recipient, value);
                    }
                }
                command_types::PAY_PORTION => {
                    type Params = sol!((address, address, uint256));
                    let (token, recipient, bips) = Params::abi_decode_params(&inputs[index], true)?;
                    if recipient != consts::MSG_SENDER && recipient != consts::FEE_COLLECTOR {
                        println!("PAY_PORTION: {:?} {:?} {:?}", token, recipient, bips);
                    }
                }
                // 0x08 <= command < 0x10
                command_types::PERMIT2_PERMIT => {
                    use Dispatcher::PermitSingle;
                    type Params = sol!((PermitSingle, bytes));
                    let (permit_single, data) = Params::abi_decode_params(&inputs[index], true)?;
                    // the token owner must be msg.sender
                }
                command_types::WRAP_ETH => {
                    type Params = sol!((address, uint256));
                    let (recipient, amount_min) = Params::abi_decode_params(&inputs[index], true)?;
                    if recipient != consts::ADDRESS_THIS {
                        println!("WRAP_ETH: {:?}", recipient);
                    }
                }
                command_types::UNWRAP_WETH => {
                    type Params = sol!((address, uint256));
                    let (recipient, amount_min) = Params::abi_decode_params(&inputs[index], true)?;
                    if recipient != consts::MSG_SENDER {
                        println!("UNWRAP_WETH: {:?}", recipient);
                    }
                }
                command_types::PERMIT2_TRANSFER_FROM_BATCH => {}
                _ => return Err(eyre!("unsupported command: {}", command)),
            }
        }
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInfo {
    pools: Vec<Address>,
    token_in: Address,
    token_out: Address,
    amount_in: U256,
    amount_out: U256,
}

enum SwapType {
    ExactIn(U256),
    ExactOut(U256),
}

fn v3_decode_swap(
    t: SwapType,
    path: &Bytes,
    traces: &Vec<TransactionTrace>,
) -> eyre::Result<SwapInfo> {
    let analyze_swap = |pool: &Address| -> eyre::Result<(bool, (U256, U256))> {
        let pool_calls = traces
            .iter()
            .filter_map(|trace| {
                let (a, i) = trace
                    .action
                    .as_call()
                    .map(|action| (&action.to, &action.input))?;
                let o = trace.result.as_ref().map(|output| output.output())?;
                Some((a, i, o))
            })
            .filter(|(addr, _, _)| **addr == *pool)
            .map(|(_, input, output)| (input, output))
            .collect::<Vec<_>>();

        if pool_calls.len() > 1 {
            return Err(eyre!("multiple pool calls not supported"));
        }
        let (input, output) = pool_calls[pool_calls.len() - 1];
        let swap_params = Dispatcher::swap_0Call::abi_decode(&input, true)?;
        type CallReturn = sol!((int256, int256));
        let amounts = CallReturn::abi_decode_params(&output, true)
            .map(|(a, b)| (a.abs().into_raw(), b.abs().into_raw()))?;
        Ok((swap_params.zeroForOne, amounts))
    };

    let paths = v3_decode_path(&path);
    let (token_in, token_out, amount_in, amount_out) = match t {
        SwapType::ExactIn(amount_in) => {
            let last_pool = paths.last().ok_or(eyre!("no path found"))?;
            let (zero_for_one, (last_token0, last_token1)) = analyze_swap(&last_pool.pool)?;
            let (token_out, amount_out) = match zero_for_one {
                true => (last_pool.token_out, last_token1),
                false => (last_pool.token_in, last_token0),
            };
            (token_out, amount_in, amount_out)
        }
        SwapType::ExactOut(amount_out) => {
            let first_path = paths.last().ok_or(eyre!("no path found"))?;
            let (zero_for_one, (first_token0, first_token1)) = analyze_swap(&first_pool.pool)?;
            let amount_in = match zero_for_one {
                true => first_token0,
                false => first_token1,
            };
            (amount_in, amount_out)
        }
    };
    println!("paths: {:?} {:?} {:?}", paths, amount_in, amount_out);
    todo!()
    // Ok(SwapInfo {
    //     pools: first_pool.pool,
    //     token_in: first_pool.token0,
    //     token_out: first_pool.token1,
    //     fee: first_pool.fee,
    //     amount_in: amount0,
    //     amount_out: amount1,
    // })
}

fn v2_decode_swap(
    path: &Vec<Address>,
    traces: &Vec<TransactionTrace>,
) -> eyre::Result<SwapInfo> {
    if path.len() > 2 {
        return Err(eyre!("multiple pools not supported"));
    }
    let token0 = path[0];
    let token1 = path[1];
    let pool = v2_compute_pool_address(token0, token1, None, None);

    let pool_calls = traces
        .iter()
        .filter_map(|trace| {
            let (a, i) = trace
                .action
                .as_call()
                .filter(|action| action.call_type == CallType::Call)
                .map(|action| (&action.to, &action.input))?;
            let o = trace.result.as_ref().map(|output| output.output())?;
            Some((a, i, o))
        })
        .filter(|(addr, _, _)| **addr == pool)
        .collect::<Vec<_>>();
    println!("{:?}", pool_calls);
    if pool_calls.len() > 1 {
        return Err(eyre!("multiple pool calls not supported"));
    }

    let (_, input, output) = pool_calls[pool_calls.len() - 1];
    let swap_call = Dispatcher::swap_1Call::abi_decode(&input, true)?;
    todo!()
    // Ok(SwapInfo {
    //     pools: pool,
    //     token_in: token0,
    //     token_out: token1,
    //     fee: 0,
    //     amount_in: I256::from_raw(swap_call.amount0Out),
    //     amount_out: I256::from_raw(swap_call.amount1Out),
    // })
}

#[derive(Debug, Clone, Copy)]
struct Pool {
    token_in: Address,
    token_out: Address,
    fee: u32,
    pool: Address,
    reverse: bool, // if true, it means token_in > token_out
}

fn v3_decode_path(path: &Bytes) -> Vec<Pool> {
    const ADDR_SIZE: usize = 20;
    const FEE_SIZE: usize = 3;

    let mut result = vec![];
    let mut offset = 0;
    loop {
        let token_in = Address::from_slice(&path.slice(offset..offset + ADDR_SIZE));
        let mut fee_buf = path.slice(offset + ADDR_SIZE..offset + FEE_SIZE + ADDR_SIZE);
        let token_out = Address::from_slice(
            &path.slice(offset + ADDR_SIZE + FEE_SIZE..offset + ADDR_SIZE * 2 + FEE_SIZE),
        );
        let fee = {
            let mut buf = [0u8; 4];
            fee_buf.copy_to_slice(&mut buf[1..]);
            let fee = u32::from_be_bytes(buf);
            fee
        };

        let pool = v3_compute_pool_address(token_in, token_out, fee, None, None);
        result.push(Pool {
            token_in,
            token_out,
            fee,
            pool,
            reverse: token_in > token_out,
        });

        offset += ADDR_SIZE + FEE_SIZE;
        if offset + ADDR_SIZE + FEE_SIZE > path.len() {
            break;
        }
    }
    result
}

fn v3_compute_pool_address(
    token_a: Address,
    token_b: Address,
    fee: u32,
    factory: Option<Address>,
    init_code_hash: Option<B256>,
) -> Address {
    assert_ne!(token_a, token_b, "TOKEN ADDRESS");
    let (token_0, token_1) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };
    let pool_key = (token_0, token_1, fee);
    factory.unwrap_or(consts::V3_FACTORY_ADDRESS).create2(
        keccak256(pool_key.abi_encode()),
        init_code_hash.unwrap_or(consts::V3_POOL_INIT_CODE_HASH),
    )
}

fn v2_compute_pool_address(
    token_a: Address,
    token_b: Address,
    factory: Option<Address>,
    init_code_hash: Option<B256>,
) -> Address {
    assert_ne!(token_a, token_b, "TOKEN ADDRESS");
    let (token_0, token_1) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };
    let pool_key = (token_0, token_1);
    factory.unwrap_or(consts::V2_FACTORY_ADDRESS).create2(
        keccak256(pool_key.abi_encode_packed()),
        init_code_hash.unwrap_or(consts::V2_PAIR_INIT_CODE_HASH),
    )
}
