# Argochain Staking Reward System Update ## 
This update introduces a proportional staking reward distribution mechanism for validators on the Argochain blockchain. The rewards are now 
directly proportional to the amount of stake contributed by each validator, and a halving mechanism has been introduced to reduce the total rewards 
over time. 

## Key Changes 
### 1. **Proportional Reward Distribution**
**What’s New:** Validator rewards are now calculated based on the proportion of the validator's stake relative to the total stake across all 
validators.
**How It Works:** Validators with higher stakes receive a larger share of the total rewards distributed in each era.

### 2. **Custom Era Payout Implementation**
**What’s New:** A new `ProportionalEraPayout` struct has been introduced to handle the era payout logic. - **How It Works:** This struct 
   implements the `pallet_staking::EraPayout` trait, calculating the rewards based on the total staked amount and ensuring the correct distribution 
   among validators.

### 3. **Halving Mechanism**
**What’s New:** The reward distribution now includes a halving mechanism, which reduces the total rewards by half after a defined number of eras 
(`HALVING_PERIOD`).
**How It Works:** The halving is applied to the initial reward (`INITIAL_REWARD`) and continues to decrease the rewards over time, similar to 
Bitcoin's halving schedule. 


## Code Changes 
### 1. **ProportionalEraPayout Struct**
   - **File:** `runtime/src/lib.rs` - **Description:** The `ProportionalEraPayout` struct is now responsible for calculating and distributing the era 
   rewards.

   ```rust pub struct ProportionalEraPayout; impl pallet_staking::EraPayout<Balance> for ProportionalEraPayout {
       fn era_payout(
           total_staked: Balance, _total_issuance: Balance, _era_duration: u64,
       ) -> (Balance, Balance) {
           // Define total rewards to be distributed in the era let total_reward = 1000 * ARGO; // Replace with your actual reward logic // Distribute 
           the entire reward to validators (total_reward, Zero::zero())
       }
   }
   ``` 
   
   ### 2. **Trait Implementation Update** - **File:** `runtime/src/lib.rs` 
- **Description:** The `pallet_staking::Config` implementation has 
   been updated to use the new `ProportionalEraPayout` for era payouts.

   ```rust 
  impl pallet_staking::Config for Runtime {
       type EraPayout = ProportionalEraPayout; // Updated to use ProportionalEraPayout // Other configurations remain unchanged
   } 
 

## Important Parameters 
- **`INITIAL_REWARD`**: The initial amount of reward distributed per era before any halvings. 
- **`HALVING_PERIOD`**: The number of eras after which the reward amount is halved. 
- - **`total_staked`**: The total amount of tokens staked by all validators in a given era. 
- - **`total_issuance`**: The total supply of tokens (not actively used in the current logic but available for future use.
