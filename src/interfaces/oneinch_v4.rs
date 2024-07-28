use alloy::{rpc::types::Transaction, sol, sol_types::SolInterface};

sol! {

interface AggregationRouterV4 {
    struct OrderRFQ {
        uint256 info;
        address makerAsset;
        address takerAsset;
        address maker;
        address allowedSender;
        uint256 makingAmount;
        uint256 takingAmount;
    }

    struct SwapDescription {
        address srcToken;
        address dstToken;
        address payable srcReceiver;
        address payable dstReceiver;
        uint256 amount;
        uint256 minReturnAmount;
        uint256 flags;
        bytes permit;
    }

    event OrderFilledRFQ(bytes32 orderHash, uint256 makingAmount);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    receive() external payable;

    function DOMAIN_SEPARATOR() external view returns (bytes32);
    function LIMIT_ORDER_RFQ_TYPEHASH() external view returns (bytes32);
    function cancelOrderRFQ(uint256 orderInfo) external;
    function clipperSwap(address srcToken, address dstToken, uint256 amount, uint256 minReturn)
        external
        payable
        returns (uint256 returnAmount);
    function clipperSwapTo(
        address payable recipient,
        address srcToken,
        address dstToken,
        uint256 amount,
        uint256 minReturn
    ) external payable returns (uint256 returnAmount);
    function clipperSwapToWithPermit(
        address payable recipient,
        address srcToken,
        address dstToken,
        uint256 amount,
        uint256 minReturn,
        bytes memory permit
    ) external returns (uint256 returnAmount);
    function destroy() external;
    function fillOrderRFQ(OrderRFQ memory order, bytes memory signature, uint256 makingAmount, uint256 takingAmount)
        external
        payable
        returns (uint256, uint256);
    function fillOrderRFQTo(
        OrderRFQ memory order,
        bytes memory signature,
        uint256 makingAmount,
        uint256 takingAmount,
        address payable target
    ) external payable returns (uint256, uint256);
    function fillOrderRFQToWithPermit(
        OrderRFQ memory order,
        bytes memory signature,
        uint256 makingAmount,
        uint256 takingAmount,
        address payable target,
        bytes memory permit
    ) external returns (uint256, uint256);
    function invalidatorForOrderRFQ(address maker, uint256 slot) external view returns (uint256);
    function owner() external view returns (address);
    function renounceOwnership() external;
    function rescueFunds(address token, uint256 amount) external;
    function swap(address caller, SwapDescription memory desc, bytes memory data)
        external
        payable
        returns (uint256 returnAmount, uint256 spentAmount, uint256 gasLeft);
    function transferOwnership(address newOwner) external;
    function uniswapV3Swap(uint256 amount, uint256 minReturn, uint256[] memory pools)
        external
        payable
        returns (uint256 returnAmount);
    function uniswapV3SwapCallback(int256 amount0Delta, int256 amount1Delta, bytes memory) external;
    function uniswapV3SwapTo(address payable recipient, uint256 amount, uint256 minReturn, uint256[] memory pools)
        external
        payable
        returns (uint256 returnAmount);
    function uniswapV3SwapToWithPermit(
        address payable recipient,
        address srcToken,
        uint256 amount,
        uint256 minReturn,
        uint256[] memory pools,
        bytes memory permit
    ) external returns (uint256 returnAmount);
    function unoswap(address srcToken, uint256 amount, uint256 minReturn, bytes32[] memory pools)
        external
        payable
        returns (uint256 returnAmount);
    function unoswapWithPermit(
        address srcToken,
        uint256 amount,
        uint256 minReturn,
        bytes32[] memory pools,
        bytes memory permit
    ) external returns (uint256 returnAmount);
}

}

// pub fn decode_swap(tx: Transaction) -> eyre::Result<()> {
//     use AggregationRouterV4::AggregationRouterV4Calls as C;

//     match C::abi_decode(&tx.input, true)? {
//         C::clipperSwap(call) => {
//             let from_address = tx.from();
//         }
//         C::clipperSwapTo(_) => todo!(),
//         C::clipperSwapToWithPermit(_) => todo!(),
//         C::fillOrderRFQ(_) => todo!(),
//         C::fillOrderRFQTo(_) => todo!(),
//         C::fillOrderRFQToWithPermit(_) => todo!(),
//         C::swap(_) => todo!(),
//         C::uniswapV3Swap(_) => todo!(),
//         C::uniswapV3SwapCallback(_) => todo!(),
//         C::uniswapV3SwapTo(_) => todo!(),
//         C::uniswapV3SwapToWithPermit(_) => todo!(),
//         C::unoswap(_) => todo!(),
//         C::unoswapWithPermit(_) => todo!(),
//         _ => {}
//     }
//     Ok(())
// }
