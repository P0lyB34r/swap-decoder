use std::io::Read;

use alloy::{
    primitives::{
        address, b256, bytes::Buf, keccak256, Address, Bytes, LogData, Signed, B256, I256, U256,
    },
    providers::network::TransactionResponse,
    rpc::types::{
        trace::parity::{Action, CallType, TraceOutput, TransactionTrace},
        Log, Transaction,
    },
    sol,
    sol_types::{SolCall, SolEvent, SolInterface, SolType, SolValue},
};
use eyre::eyre;
use serde::{Deserialize, Serialize};

use super::Decoder;

pub mod consts {
    use alloy::primitives::{address, b256, Address, B256};

    pub const NAME: &str = "Uniswap Universal Router";
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

    event Swap(
        address indexed sender,
        address indexed recipient,
        int256 amount0,
        int256 amount1,
        uint160 sqrtPriceX96,
        uint128 liquidity,
        int24 tick
    );

    event Swap(
        address indexed sender,
        uint amount0In,
        uint amount1In,
        uint amount0Out,
        uint amount1Out,
        address indexed to
    );
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
        consts::NAME.to_string()
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
        let router = context.tx().to().unwrap();

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
                        &router,
                        SwapType::ExactIn(amount_in),
                        &path,
                        &context.logs()?,
                    )?;
                    println!("V3_SWAP_EXACT_IN: {:?}", swap);
                }
                command_types::V3_SWAP_EXACT_OUT => {
                    type Params = sol!((address, uint256, uint256, bytes, bool));
                    let (recipient, amount_out, _amount_in_max, path, _payer_is_user) =
                        Params::abi_decode_params(&inputs[index], true)?;

                    let swap = v3_decode_swap(
                        &router,
                        SwapType::ExactOut(amount_out),
                        &path,
                        &context.logs()?,
                    )?;
                    println!("V3_SWAP_EXACT_OUT: {:?}", swap);
                }
                command_types::V2_SWAP_EXACT_IN => {
                    type Params = sol!((address, uint256, uint256, address[], bool));
                    let (recipient, amount_in, amount_out_min, path, payer_is_user) =
                        Params::abi_decode_params(&inputs[index], true)?;

                    let swap = v2_decode_swap(
                        &router,
                        SwapType::ExactIn(amount_in),
                        &path,
                        &context.logs()?,
                    )?;
                    println!("V2_SWAP_EXACT_IN: {:?}", swap);
                    // let swap = v2_decode_swap(&path, &context.trace().trace)?;
                }
                command_types::V2_SWAP_EXACT_OUT => {
                    type Params = sol!((address, uint256, uint256, address[], bool));
                    let (recipient, amount_out, amount_in_max, path, payer_is_user) =
                        Params::abi_decode_params(&inputs[index], true)?;

                    let swap = v2_decode_swap(
                        &router,
                        SwapType::ExactOut(amount_out),
                        &path,
                        &context.logs()?,
                    )?;
                    println!("V2_SWAP_EXACT_OUT: {:?}", swap);
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
struct SwapEntry {
    pools: Vec<Pool>,
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
    router: &Address,
    swap_type: SwapType,
    path: &Bytes,
    logs: &[Log<LogData>],
) -> eyre::Result<SwapEntry> {
    let analyze_swap = |pool: &Address| -> eyre::Result<U256> {
        let swap_logs = logs
            .iter()
            .filter(|log| log.address() == *pool && !log.removed)
            .filter_map(|log| {
                let swap = Dispatcher::Swap_0::decode_log(&log.inner, true).ok()?;
                match swap.sender == *router {
                    true => Some((swap.amount0, swap.amount1)),
                    false => None,
                }
            })
            .collect::<Vec<_>>();
        if swap_logs.len() != 1 {
            return Err(eyre!("multiple pool calls not supported"));
        }
        let (amount_0, amount_1) = swap_logs[0];
        Ok(if amount_0.is_negative() {
            amount_0.abs().into_raw()
        } else {
            amount_1.abs().into_raw()
        })
    };

    let pools = v3_decode_path(&path);
    if pools.is_empty() {
        return Err(eyre!("no path found"));
    }
    let (token_in, token_out) = (
        pools.first().unwrap().token_in,
        pools.last().unwrap().token_out,
    );
    let (amount_in, amount_out) = match swap_type {
        SwapType::ExactIn(amount_in) => {
            let amount_out = analyze_swap(&pools.last().unwrap().pool)?;
            (amount_in, amount_out)
        }
        SwapType::ExactOut(amount_out) => {
            let amount_in = analyze_swap(&pools.first().unwrap().pool)?;
            (amount_in, amount_out)
        }
    };
    Ok(SwapEntry {
        pools,
        token_in,
        token_out,
        amount_in,
        amount_out,
    })
}

fn v2_decode_swap(
    router: &Address,
    swap_type: SwapType,
    path: &Vec<Address>,
    logs: &[Log<LogData>],
) -> eyre::Result<SwapEntry> {
    let analyze_swap = |pool: &Address| -> eyre::Result<U256> {
        let swap_logs = logs
            .iter()
            .filter(|log| log.address() == *pool && !log.removed)
            .filter_map(|log| {
                let swap = Dispatcher::Swap_1::decode_log(&log.inner, true).ok()?;
                match swap.sender == *router {
                    true => Some((swap.amount0Out, swap.amount1Out)),
                    false => None,
                }
            })
            .collect::<Vec<_>>();
        if swap_logs.len() != 1 {
            return Err(eyre!("multiple pool calls not supported"));
        }
        let (amount_0, amount_1) = swap_logs[0];
        Ok(if amount_0.is_zero() {
            amount_0
        } else {
            amount_1
        })
    };

    let pools = path
        .windows(2)
        .map(|a| {
            let (token_in, token_out) = (a[0], a[1]);
            Pool {
                token_in,
                token_out,
                fee: 0,
                pool: v2_compute_pool_address(token_in, token_out, None, None),
                reverse: token_in > token_out,
            }
        })
        .collect::<Vec<_>>();
    if pools.is_empty() {
        return Err(eyre!("no path found"));
    }
    let (token_in, token_out) = (
        pools.first().unwrap().token_in,
        pools.last().unwrap().token_out,
    );
    let (amount_in, amount_out) = match swap_type {
        SwapType::ExactIn(amount_in) => {
            let amount_out = analyze_swap(&pools.last().unwrap().pool)?;
            (amount_in, amount_out)
        }
        SwapType::ExactOut(amount_out) => {
            let amount_in = analyze_swap(&pools.first().unwrap().pool)?;
            (amount_in, amount_out)
        }
    };
    Ok(SwapEntry {
        pools,
        token_in,
        token_out,
        amount_in,
        amount_out,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pool {
    token_in: Address,
    token_out: Address,
    fee: u32,
    pool: Address,
    reverse: bool, // if true, it means token_in > token_out
}

impl Pool {
    fn token_0(&self) -> Address {
        match self.reverse {
            true => self.token_out,
            false => self.token_in,
        }
    }

    fn token_1(&self) -> Address {
        match self.reverse {
            false => self.token_out,
            true => self.token_in,
        }
    }
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
