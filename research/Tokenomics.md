The equations used in Polkadot's token economics model serve to define the inflation and reward distribution based on staking dynamics. Here's a breakdown of the primary components of these equations:

1. **Inflation Rate Function (`I_NPoS(x)`):**
   The inflation function for the network is represented as `I_NPoS(x)`, where `x` represents the total staked proportion of tokens relative to the total supply. This function defines how the inflation rate changes depending on how close `x` is to the ideal staking rate (`χ_ideal`).

2. **Decay Rate (`d`):**
   The decay parameter `d` helps in determining how rapidly the inflation reduces as the staking rate moves away from `χ_ideal`. Specifically, when the staking rate (`x`) is exactly at the ideal rate, the inflation is maximized. If `x` deviates from `χ_ideal` by `d` units, the inflation rate should not decrease by more than half of its value at `χ_ideal`.

3. **Interest Rate and Inflation Rate Functions:**
   The equations model two scenarios:
   - **For `0 < x ≤ χ_ideal`**:
     \[
     i(x) = I_0 + \left(I_{NPoS}(χ_{ideal}) - I_0\right) \frac{x}{χ_{ideal}}
     \]
     This equation linearly scales the inflation rate from a base rate `I_0` up to the inflation rate at the ideal staking rate, proportional to how much `x` (the staking rate) deviates from zero up to `χ_ideal`.
   - **For `χ_ideal < x ≤ 1`**:
     \[
     i(x) = I_0 + \left(I_{NPoS}(χ_{ideal}) - I_0\right) \cdot \frac{2(χ_{ideal} - x)}{d}
     \]
     Here, the inflation decreases from the maximum value at `χ_ideal` as `x` exceeds `χ_ideal`, reducing to the base rate as `x` approaches 1. This part of the equation uses a negative slope determined by `d` to reduce the inflation rate.

4. **Output of Inflation Rate (`I_NPoS(x)/x`):**
   This term adjusts the nominal inflation rate by dividing it by the staking ratio, which normalizes the reward per staked token. It ensures that the effective per-token reward decreases as more tokens are staked, which balances the incentive for staking more tokens against the dilution of rewards as the total staked amount increases.

These mathematical models are designed to dynamically adjust the incentives for validators and nominators based on the network's staking conditions, ensuring security and participation while preventing hyperinflation or under-compensation【7†source】.