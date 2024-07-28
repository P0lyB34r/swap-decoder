use alloy::{rpc::types::Transaction, sol};

sol! {

interface TransformERC20Feature {
    struct TransformERC20Args {
        address payable taker;
        address inputToken;
        address outputToken;
        uint256 inputTokenAmount;
        uint256 minOutputTokenAmount;
        Transformation[] transformations;
        bool useSelfBalance;
        address payable recipient;
    }

    struct Transformation {
        uint32 deploymentNonce;
        bytes data;
    }

    event QuoteSignerUpdated(address quoteSigner);
    event TransformedERC20(
        address indexed taker,
        address inputToken,
        address outputToken,
        uint256 inputTokenAmount,
        uint256 outputTokenAmount
    );
    event TransformerDeployerUpdated(address transformerDeployer);

    function FEATURE_NAME() external view returns (string memory);
    function FEATURE_VERSION() external view returns (uint256);
    function _transformERC20(TransformERC20Args memory args) external payable returns (uint256 outputTokenAmount);
    function createTransformWallet() external returns (address wallet);
    function getQuoteSigner() external view returns (address signer);
    function getTransformWallet() external view returns (address wallet);
    function getTransformerDeployer() external view returns (address deployer);
    function migrate(address transformerDeployer) external returns (bytes4 success);
    function setQuoteSigner(address quoteSigner) external;
    function setTransformerDeployer(address transformerDeployer) external;
    function transformERC20(
        address inputToken,
        address outputToken,
        uint256 inputTokenAmount,
        uint256 minOutputTokenAmount,
        Transformation[] memory transformations
    ) external payable returns (uint256 outputTokenAmount);
    function transformERC20Staging(
        address inputToken,
        address outputToken,
        uint256 inputTokenAmount,
        uint256 minOutputTokenAmount,
        Transformation[] memory transformations
    ) external payable returns (uint256 outputTokenAmount);
}

}

sol! {

// Rule:
// proxiedSwap

interface ZeroExProxy {
    event AllowanceTargetChanged(address indexed allowanceTarget);
    event BeneficiaryChanged(address indexed beneficiary);
    event ImplementationOverrideSet(bytes4 indexed signature, address indexed implementation);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event ZeroExChanged(address indexed zeroEx);

    receive() external payable;

    function getAllowanceTarget() external view returns (address);
    function getBeneficiary() external view returns (address);
    function getFunctionImplementation(bytes4 signature) external returns (address impl);
    function getZeroEx() external view returns (address);
    function optimalSwap(bytes memory msgData, address feeToken, uint256 fee) external payable returns (bytes memory);
    function owner() external view returns (address);
    function proxiedSwap(
        bytes memory msgData,
        address feeToken,
        address inputToken,
        uint256 inputAmount,
        address outputToken,
        uint256 fee
    ) external payable returns (bytes memory);
    function renounceOwnership() external;
    function setAllowanceTarget(address payable newAllowanceTarget) external;
    function setBeneficiary(address payable beneficiary) external;
    function setImplementationOverride(bytes4 signature, address implementation) external;
    function setZeroEx(address newZeroEx) external;
    function transferOwnership(address newOwner) external;
}

}

pub fn get_swap_info(tx: Transaction) -> eyre::Result<()> {
    Ok(())
}