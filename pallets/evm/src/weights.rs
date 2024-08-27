// pallets/pallet-counter/src/weights.rs

use frame_support::weights::Weight;

pub trait WeightInfo {
    fn mint(amount: u64) -> Weight;
    fn burn(amount: u64) -> Weight;
    fn lock(amount: u64) -> Weight;
    fn unlock(amount: u64) -> Weight;
}

pub struct DefaultWeightInfo;

impl WeightInfo for DefaultWeightInfo {
    fn mint(amount: u64) -> Weight {
        Weight::from_parts(10_000 + 100 * amount, 0)
    }
    fn burn(amount: u64) -> Weight {
        Weight::from_parts(10_000 + 100 * amount, 0)
    }
    fn lock(amount: u64) -> Weight {
        Weight::from_parts(20_000 + 200 * amount, 0)
    }
    fn unlock(amount: u64) -> Weight {
        Weight::from_parts(20_000 + 200 * amount, 0)
    }
}
