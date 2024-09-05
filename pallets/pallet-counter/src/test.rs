use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evm_to_substrate_transfer_should_work() {
        new_test_ext().execute_with(|| {
            // Arrange
            let evm_balance = U256::from(1000u128); // The EVM balance
            let substrate_account = 1; // Assume AccountId 1 is the Substrate account
            let transfer_amount = U256::from(500u128); // Amount to transfer

            // Set up initial balance for EVM account
            EvmPallet::mutate_balance(H160::from_low_u64_be(1), evm_balance, true);

            // Act
            assert_ok!(PalletCounter::evm_to_substrate(
                Origin::signed(1), // Substrate account ID
                substrate_account,
                transfer_amount,
                false
            ));

            // Assert
            let (evm_account, _) = EvmPallet::account_basic(&H160::from_low_u64_be(1));
            assert_eq!(evm_account.balance, U256::from(500u128)); // Ensure the balance is reduced

            let substrate_balance = Balances::free_balance(&substrate_account);
            assert_eq!(substrate_balance, 500); // Check if balance has been deposited to Substrate
        });
    }

    #[test]
    fn evm_to_substrate_insufficient_balance_should_fail() {
        new_test_ext().execute_with(|| {
            // Arrange
            let evm_balance = U256::from(100u128); // Low EVM balance
            let substrate_account = 1;
            let transfer_amount = U256::from(500u128); // Amount higher than balance

            // Set up initial balance for EVM account
            EvmPallet::mutate_balance(H160::from_low_u64_be(1), evm_balance, true);

            // Act & Assert: Should fail due to insufficient balance in the EVM account
            assert_noop!(
                PalletCounter::evm_to_substrate(
                    Origin::signed(1), // Substrate account ID
                    substrate_account,
                    transfer_amount,
                    false
                ),
                Error::<Test>::InsufficientBalance
            );
        });
    }

    #[test]
    fn evm_to_substrate_amount_conversion_should_fail() {
        new_test_ext().execute_with(|| {
            // Arrange
            let evm_balance = U256::from(u128::MAX) + U256::from(1u128); // Overflow amount in U256
            let substrate_account = 1;
            let transfer_amount = evm_balance;

            // Set up initial balance for EVM account
            EvmPallet::mutate_balance(H160::from_low_u64_be(1), evm_balance, true);

            // Act & Assert: Should fail because the amount can't fit in u128
            assert_noop!(
                PalletCounter::evm_to_substrate(
                    Origin::signed(1), // Substrate account ID
                    substrate_account,
                    transfer_amount,
                    false
                ),
                Error::<Test>::AmountConversionFailed
            );
        });
    }

    #[test]
    fn evm_to_substrate_subtract_not_allowed_should_fail() {
        new_test_ext().execute_with(|| {
            // Arrange
            let evm_balance = U256::from(1000u128); // Sufficient EVM balance
            let substrate_account = 1;
            let transfer_amount = U256::from(500u128); // Valid amount

            // Set up initial balance for EVM account
            EvmPallet::mutate_balance(H160::from_low_u64_be(1), evm_balance, true);

            // Act & Assert: Should fail due to subtract being set to true (which is not allowed)
            assert_noop!(
                PalletCounter::evm_to_substrate(
                    Origin::signed(1), // Substrate account ID
                    substrate_account,
                    transfer_amount,
                    true // Subtraction is not allowed
                ),
                Error::<Test>::OperationNotAllowed
            );
        });
    }
}
