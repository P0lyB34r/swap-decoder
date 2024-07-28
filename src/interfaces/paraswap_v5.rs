use alloy::{rpc::types::Transaction, sol, sol_types::SolInterface};

sol! {

interface MultiPath {
    struct Adapter {
        address payable adapter;
        uint256 percent;
        uint256 networkFee;
        Route[] route;
    }

    struct BuyData {
        address adapter;
        address fromToken;
        address toToken;
        uint256 fromAmount;
        uint256 toAmount;
        uint256 expectedAmount;
        address payable beneficiary;
        Route[] route;
        address payable partner;
        uint256 feePercent;
        bytes permit;
        uint256 deadline;
        bytes16 uuid;
    }

    struct MegaSwapPath {
        uint256 fromAmountPercent;
        Path[] path;
    }

    struct MegaSwapSellData {
        address fromToken;
        uint256 fromAmount;
        uint256 toAmount;
        uint256 expectedAmount;
        address payable beneficiary;
        MegaSwapPath[] path;
        address payable partner;
        uint256 feePercent;
        bytes permit;
        uint256 deadline;
        bytes16 uuid;
    }

    struct Path {
        address to;
        uint256 totalNetworkFee;
        Adapter[] adapters;
    }

    struct Route {
        uint256 index;
        address targetExchange;
        uint256 percent;
        bytes payload;
        uint256 networkFee;
    }

    struct SellData {
        address fromToken;
        uint256 fromAmount;
        uint256 toAmount;
        uint256 expectedAmount;
        address payable beneficiary;
        Path[] path;
        address payable partner;
        uint256 feePercent;
        bytes permit;
        uint256 deadline;
        bytes16 uuid;
    }

    event BoughtV3(
        bytes16 uuid,
        address partner,
        uint256 feePercent,
        address initiator,
        address indexed beneficiary,
        address indexed srcToken,
        address indexed destToken,
        uint256 srcAmount,
        uint256 receivedAmount,
        uint256 expectedAmount
    );
    event SwappedV3(
        bytes16 uuid,
        address partner,
        uint256 feePercent,
        address initiator,
        address indexed beneficiary,
        address indexed srcToken,
        address indexed destToken,
        uint256 srcAmount,
        uint256 receivedAmount,
        uint256 expectedAmount
    );

    function ROUTER_ROLE() external view returns (bytes32);
    function WHITELISTED_ROLE() external view returns (bytes32);
    function buy(BuyData memory data) external payable returns (uint256);
    function feeClaimer() external view returns (address);
    function getKey() external pure returns (bytes32);
    function initialize(bytes memory) external;
    function maxFeePercent() external view returns (uint256);
    function megaSwap(MegaSwapSellData memory data) external payable returns (uint256);
    function multiSwap(SellData memory data) external payable returns (uint256);
    function paraswapReferralShare() external view returns (uint256);
    function paraswapSlippageShare() external view returns (uint256);
    function partnerSharePercent() external view returns (uint256);
}

}

sol! {

interface DirectSwap {
    type CurveSwapType is uint8;
    type DirectSwapKind is uint8;

    struct BatchSwapStep {
        bytes32 poolId;
        uint256 assetInIndex;
        uint256 assetOutIndex;
        uint256 amount;
        bytes userData;
    }

    struct DirectBalancerV2 {
        BatchSwapStep[] swaps;
        address[] assets;
        FundManagement funds;
        int256[] limits;
        uint256 fromAmount;
        uint256 toAmount;
        uint256 expectedAmount;
        uint256 deadline;
        uint256 feePercent;
        address vault;
        address payable partner;
        bool isApproved;
        address payable beneficiary;
        bytes permit;
        bytes16 uuid;
    }

    struct DirectCurveV1 {
        address fromToken;
        address toToken;
        address exchange;
        uint256 fromAmount;
        uint256 toAmount;
        uint256 expectedAmount;
        uint256 feePercent;
        int128 i;
        int128 j;
        address payable partner;
        bool isApproved;
        CurveSwapType swapType;
        address payable beneficiary;
        bool needWrapNative;
        bytes permit;
        bytes16 uuid;
    }

    struct DirectCurveV2 {
        address fromToken;
        address toToken;
        address exchange;
        address poolAddress;
        uint256 fromAmount;
        uint256 toAmount;
        uint256 expectedAmount;
        uint256 feePercent;
        uint256 i;
        uint256 j;
        address payable partner;
        bool isApproved;
        CurveSwapType swapType;
        address payable beneficiary;
        bool needWrapNative;
        bytes permit;
        bytes16 uuid;
    }

    struct DirectUniV3 {
        address fromToken;
        address toToken;
        address exchange;
        uint256 fromAmount;
        uint256 toAmount;
        uint256 expectedAmount;
        uint256 feePercent;
        uint256 deadline;
        address payable partner;
        bool isApproved;
        address payable beneficiary;
        bytes path;
        bytes permit;
        bytes16 uuid;
    }

    struct FundManagement {
        address sender;
        bool fromInternalBalance;
        address payable recipient;
        bool toInternalBalance;
    }

    event BoughtV3(
        bytes16 uuid,
        address partner,
        uint256 feePercent,
        address initiator,
        address indexed beneficiary,
        address indexed srcToken,
        address indexed destToken,
        uint256 srcAmount,
        uint256 receivedAmount,
        uint256 expectedAmount
    );
    event SwappedDirect(
        bytes16 uuid,
        address partner,
        uint256 feePercent,
        address initiator,
        DirectSwapKind kind,
        address indexed beneficiary,
        address indexed srcToken,
        address indexed destToken,
        uint256 srcAmount,
        uint256 receivedAmount,
        uint256 expectedAmount
    );
    event SwappedV3(
        bytes16 uuid,
        address partner,
        uint256 feePercent,
        address initiator,
        address indexed beneficiary,
        address indexed srcToken,
        address indexed destToken,
        uint256 srcAmount,
        uint256 receivedAmount,
        uint256 expectedAmount
    );

    function ROUTER_ROLE() external view returns (bytes32);
    function WHITELISTED_ROLE() external view returns (bytes32);
    function directBalancerV2GivenInSwap(DirectBalancerV2 memory data) external payable;
    function directBalancerV2GivenOutSwap(DirectBalancerV2 memory data) external payable;
    function directCurveV1Swap(DirectCurveV1 memory data) external payable;
    function directCurveV2Swap(DirectCurveV2 memory data) external payable;
    function directUniV3Buy(DirectUniV3 memory data) external payable;
    function directUniV3Swap(DirectUniV3 memory data) external payable;
    function feeClaimer() external view returns (address);
    function getKey() external pure returns (bytes32);
    function initialize(bytes memory) external pure;
    function maxFeePercent() external view returns (uint256);
    function paraswapReferralShare() external view returns (uint256);
    function paraswapSlippageShare() external view returns (uint256);
    function partnerSharePercent() external view returns (uint256);
    function weth() external view returns (address);
}

}
