// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Auxiliary `struct`/`enum`s for polkadot runtime.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{
	fungible::{Balanced, Credit},
	tokens::imbalance::ResolveTo,
	Contains, ContainsPair, Imbalance, OnUnbalanced,
};
use pallet_treasury::TreasuryAccountId;
use polkadot_primitives::Balance;
use sp_runtime::{traits::TryConvert, Perquintill, RuntimeDebug};
use xcm::VersionedLocation;

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(core::marker::PhantomData<R>);
impl<R> OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>> for ToAuthor<R>
where
	R: pallet_balances::Config + pallet_authorship::Config,
	<R as frame_system::Config>::AccountId: From<polkadot_primitives::AccountId>,
	<R as frame_system::Config>::AccountId: Into<polkadot_primitives::AccountId>,
{
	fn on_nonzero_unbalanced(
		amount: Credit<<R as frame_system::Config>::AccountId, pallet_balances::Pallet<R>>,
	) {
		if let Some(author) = <pallet_authorship::Pallet<R>>::author() {
			let _ = <pallet_balances::Pallet<R>>::resolve(&author, amount);
		}
	}
}

pub struct DealWithFees<R>(core::marker::PhantomData<R>);
impl<R> OnUnbalanced<Credit<R::AccountId, pallet_balances::Pallet<R>>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_authorship::Config + pallet_treasury::Config,
	<R as frame_system::Config>::AccountId: From<polkadot_primitives::AccountId>,
	<R as frame_system::Config>::AccountId: Into<polkadot_primitives::AccountId>,
{
	fn on_unbalanceds<B>(
		mut fees_then_tips: impl Iterator<Item = Credit<R::AccountId, pallet_balances::Pallet<R>>>,
	) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 80% to treasury, 20% to author
			let mut split = fees.ration(80, 20);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 100% to author
				tips.merge_into(&mut split.1);
			}
			ResolveTo::<TreasuryAccountId<R>, pallet_balances::Pallet<R>>::on_unbalanced(split.0);
			<ToAuthor<R> as OnUnbalanced<_>>::on_unbalanced(split.1);
		}
	}
}

pub fn era_payout(
	total_staked: Balance,
	total_stakable: Balance,
	max_annual_inflation: Perquintill,
	period_fraction: Perquintill,
	auctioned_slots: u64,
) -> (Balance, Balance) {
	use pallet_staking_reward_fn::compute_inflation;
	use sp_runtime::traits::Saturating;

	let min_annual_inflation = Perquintill::from_rational(25u64, 1000u64);
	let delta_annual_inflation = max_annual_inflation.saturating_sub(min_annual_inflation);

	// 30% reserved for up to 60 slots.
	let auction_proportion = Perquintill::from_rational(auctioned_slots.min(60), 200u64);

	// Therefore the ideal amount at stake (as a percentage of total issuance) is 75% less the
	// amount that we expect to be taken up with auctions.
	let ideal_stake = Perquintill::from_percent(75).saturating_sub(auction_proportion);

	let stake = Perquintill::from_rational(total_staked, total_stakable);
	let falloff = Perquintill::from_percent(5);
	let adjustment = compute_inflation(stake, ideal_stake, falloff);
	let staking_inflation =
		min_annual_inflation.saturating_add(delta_annual_inflation * adjustment);

	let max_payout = period_fraction * max_annual_inflation * total_stakable;
	let staking_payout = (period_fraction * staking_inflation) * total_stakable;
	let rest = max_payout.saturating_sub(staking_payout);

	let other_issuance = total_stakable.saturating_sub(total_staked);
	if total_staked > other_issuance {
		let _cap_rest = Perquintill::from_rational(other_issuance, total_staked) * staking_payout;
		// We don't do anything with this, but if we wanted to, we could introduce a cap on the
		// treasury amount with: `rest = rest.min(cap_rest);`
	}
	(staking_payout, rest)
}

/// Versioned locatable asset type which contains both an XCM `location` and `asset_id` to identify
/// an asset which exists on some chain.
#[derive(
	Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, scale_info::TypeInfo, MaxEncodedLen,
)]
pub enum VersionedLocatableAsset {
	#[codec(index = 3)]
	V3 { location: xcm::v3::Location, asset_id: xcm::v3::AssetId },
	#[codec(index = 4)]
	V4 { location: xcm::v4::Location, asset_id: xcm::v4::AssetId },
}

/// Converts the [`VersionedLocatableAsset`] to the [`xcm_builder::LocatableAssetId`].
pub struct LocatableAssetConverter;
impl TryConvert<VersionedLocatableAsset, xcm_builder::LocatableAssetId>
	for LocatableAssetConverter
{
	fn try_convert(
		asset: VersionedLocatableAsset,
	) -> Result<xcm_builder::LocatableAssetId, VersionedLocatableAsset> {
		match asset {
			VersionedLocatableAsset::V3 { location, asset_id } =>
				Ok(xcm_builder::LocatableAssetId {
					location: location.try_into().map_err(|_| asset.clone())?,
					asset_id: asset_id.try_into().map_err(|_| asset.clone())?,
				}),
			VersionedLocatableAsset::V4 { location, asset_id } =>
				Ok(xcm_builder::LocatableAssetId { location, asset_id }),
		}
	}
}

/// Converts the [`VersionedLocation`] to the [`xcm::latest::Location`].
pub struct VersionedLocationConverter;
impl TryConvert<&VersionedLocation, xcm::latest::Location> for VersionedLocationConverter {
	fn try_convert(
		location: &VersionedLocation,
	) -> Result<xcm::latest::Location, &VersionedLocation> {
		let latest = match location.clone() {
			VersionedLocation::V2(l) => {
				let v3: xcm::v3::Location = l.try_into().map_err(|_| location)?;
				v3.try_into().map_err(|_| location)?
			},
			VersionedLocation::V3(l) => l.try_into().map_err(|_| location)?,
			VersionedLocation::V4(l) => l,
		};
		Ok(latest)
	}
}

/// Adapter for [`Contains`] trait to match [`VersionedLocatableAsset`] type converted to the latest
/// version of itself where it's location matched by `L` and it's asset id by `A` parameter types.
pub struct ContainsParts<C>(core::marker::PhantomData<C>);
impl<C> Contains<VersionedLocatableAsset> for ContainsParts<C>
where
	C: ContainsPair<xcm::latest::Location, xcm::latest::Location>,
{
	fn contains(asset: &VersionedLocatableAsset) -> bool {
		use VersionedLocatableAsset::*;
		let (location, asset_id) = match asset.clone() {
			V3 { location, asset_id } => match (location.try_into(), asset_id.try_into()) {
				(Ok(l), Ok(a)) => (l, a),
				_ => return false,
			},
			V4 { location, asset_id } => (location, asset_id),
		};
		C::contains(&location, &asset_id.0)
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarks {
	use super::VersionedLocatableAsset;
	use core::marker::PhantomData;
	use frame_support::traits::Get;
	use pallet_asset_rate::AssetKindFactory;
	use pallet_treasury::ArgumentsFactory as TreasuryArgumentsFactory;
	use sp_core::{ConstU32, ConstU8};
	use xcm::prelude::*;

	/// Provides a factory method for the [`VersionedLocatableAsset`].
	/// The location of the asset is determined as a Parachain with an ID equal to the passed seed.
	pub struct AssetRateArguments;
	impl AssetKindFactory<VersionedLocatableAsset> for AssetRateArguments {
		fn create_asset_kind(seed: u32) -> VersionedLocatableAsset {
			VersionedLocatableAsset::V4 {
				location: xcm::v4::Location::new(0, [xcm::v4::Junction::Parachain(seed)]),
				asset_id: xcm::v4::Location::new(
					0,
					[
						xcm::v4::Junction::PalletInstance(seed.try_into().unwrap()),
						xcm::v4::Junction::GeneralIndex(seed.into()),
					],
				)
				.into(),
			}
		}
	}

	/// Provide factory methods for the [`VersionedLocatableAsset`] and the `Beneficiary` of the
	/// [`VersionedLocation`]. The location of the asset is determined as a Parachain with an
	/// ID equal to the passed seed.
	pub struct TreasuryArguments<Parents = ConstU8<0>, ParaId = ConstU32<0>>(
		PhantomData<(Parents, ParaId)>,
	);
	impl<Parents: Get<u8>, ParaId: Get<u32>>
		TreasuryArgumentsFactory<VersionedLocatableAsset, VersionedLocation>
		for TreasuryArguments<Parents, ParaId>
	{
		fn create_asset_kind(seed: u32) -> VersionedLocatableAsset {
			VersionedLocatableAsset::V3 {
				location: xcm::v3::Location::new(
					Parents::get(),
					[xcm::v3::Junction::Parachain(ParaId::get())],
				),
				asset_id: xcm::v3::Location::new(
					0,
					[
						xcm::v3::Junction::PalletInstance(seed.try_into().unwrap()),
						xcm::v3::Junction::GeneralIndex(seed.into()),
					],
				)
				.into(),
			}
		}
		fn create_beneficiary(seed: [u8; 32]) -> VersionedLocation {
			VersionedLocation::V4(xcm::v4::Location::new(
				0,
				[xcm::v4::Junction::AccountId32 { network: None, id: seed }],
			))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{
		derive_impl,
		dispatch::DispatchClass,
		parameter_types,
		traits::{
			tokens::{PayFromAccount, UnityAssetBalanceConversion},
			FindAuthor,
		},
		weights::Weight,
		PalletId,
	};
	use frame_system::limits;
	use polkadot_primitives::AccountId;
	use sp_core::{ConstU64, H256};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup},
		BuildStorage, Perbill,
	};

	type Block = frame_system::mocking::MockBlock<Test>;
	const TEST_ACCOUNT: AccountId = AccountId::new([1; 32]);

	frame_support::construct_runtime!(
		pub enum Test
		{
			System: frame_system,
			Authorship: pallet_authorship,
			Balances: pallet_balances,
			Treasury: pallet_treasury,
		}
	);

	parameter_types! {
		pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
			.base_block(Weight::from_parts(10, 0))
			.for_class(DispatchClass::all(), |weight| {
				weight.base_extrinsic = Weight::from_parts(100, 0);
			})
			.for_class(DispatchClass::non_mandatory(), |weight| {
				weight.max_total = Some(Weight::from_parts(1024, u64::MAX));
			})
			.build_or_panic();
		pub BlockLength: limits::BlockLength = limits::BlockLength::max(2 * 1024);
		pub const AvailableBlockRatio: Perbill = Perbill::one();
	}

	#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
	impl frame_system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type RuntimeOrigin = RuntimeOrigin;
		type Nonce = u64;
		type RuntimeCall = RuntimeCall;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Block = Block;
		type RuntimeEvent = RuntimeEvent;
		type BlockLength = BlockLength;
		type BlockWeights = BlockWeights;
		type DbWeight = ();
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<u64>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
	impl pallet_balances::Config for Test {
		type AccountStore = System;
	}

	parameter_types! {
		pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
		pub const MaxApprovals: u32 = 100;
		pub TreasuryAccount: AccountId = Treasury::account_id();
	}

	impl pallet_treasury::Config for Test {
		type Currency = pallet_balances::Pallet<Test>;
		type RejectOrigin = frame_system::EnsureRoot<AccountId>;
		type RuntimeEvent = RuntimeEvent;
		type SpendPeriod = ();
		type Burn = ();
		type BurnDestination = ();
		type PalletId = TreasuryPalletId;
		type SpendFunds = ();
		type MaxApprovals = MaxApprovals;
		type WeightInfo = ();
		type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u64>;
		type AssetKind = ();
		type Beneficiary = Self::AccountId;
		type BeneficiaryLookup = IdentityLookup<Self::AccountId>;
		type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
		type BalanceConverter = UnityAssetBalanceConversion;
		type PayoutPeriod = ConstU64<0>;
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper = ();
	}

	pub struct OneAuthor;
	impl FindAuthor<AccountId> for OneAuthor {
		fn find_author<'a, I>(_: I) -> Option<AccountId>
		where
			I: 'a,
		{
			Some(TEST_ACCOUNT)
		}
	}
	impl pallet_authorship::Config for Test {
		type FindAuthor = OneAuthor;
		type EventHandler = ();
	}

	pub fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		// We use default for brevity, but you can configure as desired if needed.
		pallet_balances::GenesisConfig::<Test>::default()
			.assimilate_storage(&mut t)
			.unwrap();
		t.into()
	}

	#[test]
	fn test_fees_and_tip_split() {
		new_test_ext().execute_with(|| {
			let fee =
				<pallet_balances::Pallet<Test> as frame_support::traits::fungible::Balanced<
					AccountId,
				>>::issue(10);
			let tip =
				<pallet_balances::Pallet<Test> as frame_support::traits::fungible::Balanced<
					AccountId,
				>>::issue(20);

			assert_eq!(Balances::free_balance(Treasury::account_id()), 0);
			assert_eq!(Balances::free_balance(TEST_ACCOUNT), 0);

			DealWithFees::on_unbalanceds(vec![fee, tip].into_iter());

			// Author gets 100% of tip and 20% of fee = 22
			assert_eq!(Balances::free_balance(TEST_ACCOUNT), 22);
			// Treasury gets 80% of fee
			assert_eq!(Balances::free_balance(Treasury::account_id()), 8);
		});
	}

	#[test]
	fn compute_inflation_should_give_sensible_results() {
		assert_eq!(
			pallet_staking_reward_fn::compute_inflation(
				Perquintill::from_percent(75),
				Perquintill::from_percent(75),
				Perquintill::from_percent(5),
			),
			Perquintill::one()
		);
		assert_eq!(
			pallet_staking_reward_fn::compute_inflation(
				Perquintill::from_percent(50),
				Perquintill::from_percent(75),
				Perquintill::from_percent(5),
			),
			Perquintill::from_rational(2u64, 3u64)
		);
		assert_eq!(
			pallet_staking_reward_fn::compute_inflation(
				Perquintill::from_percent(80),
				Perquintill::from_percent(75),
				Perquintill::from_percent(5),
			),
			Perquintill::from_rational(1u64, 2u64)
		);
	}

	#[test]
	fn era_payout_should_give_sensible_results() {
		assert_eq!(
			era_payout(75, 100, Perquintill::from_percent(10), Perquintill::one(), 0,),
			(10, 0)
		);
		assert_eq!(
			era_payout(80, 100, Perquintill::from_percent(10), Perquintill::one(), 0,),
			(6, 4)
		);
	}
}
