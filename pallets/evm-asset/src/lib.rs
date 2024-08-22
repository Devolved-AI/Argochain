#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
pub use pallet::*;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::HasCompact;
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);
	pub const PALLET_ID: PalletId = PalletId(*b"asstmngr");
	pub trait AssetRegistrar<T: Config> {
		fn create_foreign_asset(
			_asset: T::AssetId,
			_min_balance: T::Balance,
			_metadata: T::AssetRegistrarMetadata,
			_is_sufficient: bool,
		) -> DispatchResult {
			unimplemented!()
		}

		fn destroy_foreign_asset(_asset: T::AssetId) -> DispatchResult {
			unimplemented!()
		}

		fn destroy_asset_dispatch_info_weight(_asset: T::AssetId) -> Weight;
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type AssetId: Member + Parameter + Default + Copy + HasCompact + MaxEncodedLen;
		type AssetRegistrarMetadata: Member + Parameter + Default;
		type ForeignAssetType: Parameter + Member + Ord + PartialOrd + Into<Self::AssetId> + Default;
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		type AssetRegistrar: AssetRegistrar<Self>;
		type ForeignAssetModifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::event]
	pub enum Event<T: Config> {}

	#[pallet::storage]
	#[pallet::getter(fn asset_id_type)]
	pub type AssetIdType<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::ForeignAssetType>;

	#[pallet::storage]
	#[pallet::getter(fn asset_type_id)]
	pub type AssetTypeId<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ForeignAssetType, T::AssetId>;

	#[pallet::storage]
	#[pallet::getter(fn asset_type_units_per_second)]
	pub type AssetTypeUnitsPerSecond<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ForeignAssetType, u128>;

	#[pallet::storage]
	#[pallet::getter(fn supported_fee_payment_assets)]
	pub type SupportedFeePaymentAssets<T: Config> =
		StorageValue<_, Vec<T::ForeignAssetType>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::register_foreign_asset())]
		pub fn register_foreign_asset(
			origin: OriginFor<T>,
			asset: T::ForeignAssetType,
			metadata: T::AssetRegistrarMetadata,
			min_amount: T::Balance,
			is_sufficient: bool,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;
			let asset_id: T::AssetId = asset.clone().into();
			ensure!(
				AssetIdType::<T>::get(&asset_id).is_none(),
				Error::<T>::AssetAlreadyExists
			);
			T::AssetRegistrar::create_foreign_asset(
				asset_id,
				min_amount,
				metadata.clone(),
				is_sufficient,
			)
			.map_err(|_| Error::<T>::ErrorCreatingAsset)?;

			AssetIdType::<T>::insert(&asset_id, &asset);
			AssetTypeId::<T>::insert(&asset, &asset_id);

			Self::deposit_event(Event::ForeignAssetRegistered {
				asset_id,
				asset,
				metadata,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::change_existing_asset_type(*num_assets_weight_hint))]
		pub fn change_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			new_asset_type: T::ForeignAssetType,
			num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;
			let mut supported_assets = SupportedFeePaymentAssets::<T>::get();

			ensure!(
				num_assets_weight_hint >= (supported_assets.len() as u32),
				Error::<T>::TooLowNumAssetsWeightHint
			);

			let previous_asset_type =
				AssetIdType::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			AssetIdType::<T>::insert(&asset_id, &new_asset_type);
			AssetTypeId::<T>::insert(&new_asset_type, &asset_id);
			AssetTypeId::<T>::remove(&previous_asset_type);
			if let Some(units) = AssetTypeUnitsPerSecond::<T>::get(&previous_asset_type) {
				if let Ok(index) = supported_assets.binary_search(&previous_asset_type) {
					supported_assets.remove(index);
				}
				if let Err(index) = supported_assets.binary_search(&new_asset_type) {
					supported_assets.insert(index, new_asset_type.clone());
				}
				SupportedFeePaymentAssets::<T>::put(supported_assets);
				AssetTypeUnitsPerSecond::<T>::remove(&previous_asset_type);
				AssetTypeUnitsPerSecond::<T>::insert(&new_asset_type, units);
			}
			Self::deposit_event(Event::ForeignAssetXcmLocationChanged {
				asset_id,
				new_asset_type,
			});
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::remove_supported_asset(*num_assets_weight_hint))]
		pub fn remove_supported_asset(
			origin: OriginFor<T>,
			asset_type: T::ForeignAssetType,
			num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;
			let mut supported_assets = SupportedFeePaymentAssets::<T>::get();
			ensure!(
				num_assets_weight_hint >= (supported_assets.len() as u32),
				Error::<T>::TooLowNumAssetsWeightHint
			);
			if let Ok(index) = supported_assets.binary_search(&asset_type) {
				supported_assets.remove(index);
			}
			SupportedFeePaymentAssets::<T>::put(supported_assets);
			AssetTypeUnitsPerSecond::<T>::remove(&asset_type);
			Self::deposit_event(Event::SupportedAssetRemoved { asset_type });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			PALLET_ID.into_account_truncating()
		}
	}
}
