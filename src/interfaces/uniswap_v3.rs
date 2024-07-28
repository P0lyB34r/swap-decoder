use alloy::{
    primitives::address, rpc::types::{Transaction, TransactionReceipt}, sol, sol_types::{SolCall, SolInterface}
};

use super::{get_tx_receipt, get_tx_trace, Decoder};

sol! {

interface SwapRouter02 {
    type ApprovalType is uint8;

    struct ExactInputParams {
        bytes path;
        address recipient;
        uint256 amountIn;
        uint256 amountOutMinimum;
    }

    struct ExactInputSingleParams {
        address tokenIn;
        address tokenOut;
        uint24 fee;
        address recipient;
        uint256 amountIn;
        uint256 amountOutMinimum;
        uint160 sqrtPriceLimitX96;
    }

    struct ExactOutputParams {
        bytes path;
        address recipient;
        uint256 amountOut;
        uint256 amountInMaximum;
    }

    struct ExactOutputSingleParams {
        address tokenIn;
        address tokenOut;
        uint24 fee;
        address recipient;
        uint256 amountOut;
        uint256 amountInMaximum;
        uint160 sqrtPriceLimitX96;
    }

    struct IncreaseLiquidityParams {
        address token0;
        address token1;
        uint256 tokenId;
        uint256 amount0Min;
        uint256 amount1Min;
    }

    struct MintParams {
        address token0;
        address token1;
        uint24 fee;
        int24 tickLower;
        int24 tickUpper;
        uint256 amount0Min;
        uint256 amount1Min;
        address recipient;
    }

    receive() external payable;

    function WETH9() external view returns (address);
    function approveMax(address token) external payable;
    function approveMaxMinusOne(address token) external payable;
    function approveZeroThenMax(address token) external payable;
    function approveZeroThenMaxMinusOne(address token) external payable;
    function callPositionManager(bytes memory data) external payable returns (bytes memory result);
    function checkOracleSlippage(
        bytes[] memory paths,
        uint128[] memory amounts,
        uint24 maximumTickDivergence,
        uint32 secondsAgo
    ) external view;
    function checkOracleSlippage(bytes memory path, uint24 maximumTickDivergence, uint32 secondsAgo) external view;
    function exactInput(ExactInputParams memory params) external payable returns (uint256 amountOut);
    function exactInputSingle(ExactInputSingleParams memory params) external payable returns (uint256 amountOut);
    function exactOutput(ExactOutputParams memory params) external payable returns (uint256 amountIn);
    function exactOutputSingle(ExactOutputSingleParams memory params) external payable returns (uint256 amountIn);
    function factory() external view returns (address);
    function factoryV2() external view returns (address);
    function getApprovalType(address token, uint256 amount) external returns (ApprovalType);
    function increaseLiquidity(IncreaseLiquidityParams memory params) external payable returns (bytes memory result);
    function mint(MintParams memory params) external payable returns (bytes memory result);
    function multicall(bytes32 previousBlockhash, bytes[] memory data) external payable returns (bytes[] memory);
    function multicall(uint256 deadline, bytes[] memory data) external payable returns (bytes[] memory);
    function multicall(bytes[] memory data) external payable returns (bytes[] memory results);
    function positionManager() external view returns (address);
    function pull(address token, uint256 value) external payable;
    function refundETH() external payable;
    function selfPermit(address token, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
        external
        payable;
    function selfPermitAllowed(address token, uint256 nonce, uint256 expiry, uint8 v, bytes32 r, bytes32 s)
        external
        payable;
    function selfPermitAllowedIfNecessary(address token, uint256 nonce, uint256 expiry, uint8 v, bytes32 r, bytes32 s)
        external
        payable;
    function selfPermitIfNecessary(address token, uint256 value, uint256 deadline, uint8 v, bytes32 r, bytes32 s)
        external
        payable;
    function swapExactTokensForTokens(uint256 amountIn, uint256 amountOutMin, address[] memory path, address to)
        external
        payable
        returns (uint256 amountOut);
    function swapTokensForExactTokens(uint256 amountOut, uint256 amountInMax, address[] memory path, address to)
        external
        payable
        returns (uint256 amountIn);
    function sweepToken(address token, uint256 amountMinimum, address recipient) external payable;
    function sweepToken(address token, uint256 amountMinimum) external payable;
    function sweepTokenWithFee(address token, uint256 amountMinimum, uint256 feeBips, address feeRecipient)
        external
        payable;
    function sweepTokenWithFee(
        address token,
        uint256 amountMinimum,
        address recipient,
        uint256 feeBips,
        address feeRecipient
    ) external payable;
    function uniswapV3SwapCallback(int256 amount0Delta, int256 amount1Delta, bytes memory _data) external;
    function unwrapWETH9(uint256 amountMinimum, address recipient) external payable;
    function unwrapWETH9(uint256 amountMinimum) external payable;
    function unwrapWETH9WithFee(uint256 amountMinimum, address recipient, uint256 feeBips, address feeRecipient)
        external
        payable;
    function unwrapWETH9WithFee(uint256 amountMinimum, uint256 feeBips, address feeRecipient) external payable;
    function wrapETH(uint256 value) external payable;
}

}

pub struct DecoderUniswapV3 {}

impl Decoder for DecoderUniswapV3 {
    fn name(&self) -> String {
        "UniswapV3".to_string()
    }

    fn supported_address(&self) -> Vec<alloy::primitives::Address> {
        // vec![address!("")]
        todo!()
    }

    fn supported_selectors(&self) -> Vec<[u8; 4]> {
        use SwapRouter02 as C;
        vec![
            C::exactInputCall::SELECTOR,
            C::exactInputSingleCall::SELECTOR,
            C::exactOutputCall::SELECTOR,
            C::exactOutputSingleCall::SELECTOR,
            C::multicall_0Call::SELECTOR,
            C::multicall_1Call::SELECTOR,
            C::multicall_2Call::SELECTOR,
        ]
    }

    fn decode(&self, context: &super::DecoderContext) -> eyre::Result<super::Swap> {
        todo!()
    }
}

// pub fn get_swap_info(tx: Transaction) -> eyre::Result<()> {
//     use SwapRouter02::SwapRouter02Calls as C;

//     let call_dispatch = C::abi_decode(&tx.input, true)?;
//     let receipt = futures::executor::block_on(get_tx_receipt(url, &tx.hash))?;
//     let trace = futures::executor::block_on(get_tx_trace(url, &tx.hash))?;

//     match call_dispatch {
//         C::exactInput(call) => {}
//         C::exactInputSingle(_) => todo!(),
//         C::exactOutput(_) => todo!(),
//         C::exactOutputSingle(_) => todo!(),
//         C::multicall_0(_) => todo!(),
//         C::multicall_1(_) => todo!(),
//         C::multicall_2(_) => todo!(),
//         _ => unimplemented!("Uniswap V3"),
//     }
//     Ok(())
// }
