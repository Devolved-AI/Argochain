#[test]
fn minting_with_sudo_account_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(MintPallet::mint_coins(Origin::root(), 1, 1000));
        assert_eq!(Balances::free_balance(1), 1000);
    });
}

#[test]
fn minting_with_non_sudo_account_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(MintPallet::mint_coins(Origin::signed(1), 2, 1000), Error::<Test>::NotSudo);
    });
}
