# Token Economics of Argochain

## Inflation model

Token economics models serve to define the inflation and reward distribution based on staking dynamics. Here's a breakdown of the primary components of these equations:

1. **Inflation Rate Function (`I_NPoS(x)`):**
   The inflation function for the network is represented as `I_NPoS(x)`, where `x` represents the total staked proportion of tokens relative to the total supply. This function defines how the inflation rate changes depending on how close `x` is to the ideal staking rate (`χ_ideal`).

2. **Decay Rate (`d`):**
   The decay parameter `d` helps in determining how rapidly the inflation reduces as the staking rate moves away from `χ_ideal`. Specifically, when the staking rate (`x`) is exactly at the ideal rate, the inflation is maximized. If `x` deviates from `χ_ideal` by `d` units, the inflation rate should not decrease by more than half of its value at `χ_ideal`.

3. **Interest Rate and Inflation Rate Functions:**
   The equations model two scenarios:
   - **For `0 < x ≤ χ_ideal`**:
      ```
      i(x) = I_0 + (I_NPoS * (χ_ideal) - I_0) * (x /  χ_ideal)
      ```
     This equation linearly scales the inflation rate from a base rate `I_0` up to the inflation rate at the ideal staking rate, proportional to how much `x` (the staking rate) deviates from zero up to `χ_ideal`.
   - **For `χ_ideal < x ≤ 1`**:
     ```
     i(x) = I_0 + (I_NPoS * (χ_ideal) - I_0) * (2 * (χ_ideal - x) / d)
     ```
     Here, the inflation decreases from the maximum value at `χ_ideal` as `x` exceeds `χ_ideal`, reducing to the base rate as `x` approaches 1. This part of the equation uses a negative slope determined by `d` to reduce the inflation rate.

4. **Output of Inflation Rate (`I_NPoS(x)/x`):**
   This term adjusts the nominal inflation rate by dividing it by the staking ratio, which normalizes the reward per staked token. It ensures that the effective per-token reward decreases as more tokens are staked, which balances the incentive for staking more tokens against the dilution of rewards as the total staked amount increases.

These mathematical models are designed to dynamically adjust the incentives for validators and nominators based on the network's staking conditions, ensuring security and participation while preventing hyperinflation or under-compensation.



## Calculating Coins per Block

1. **Annual Inflation Rate**: Based on your setup, the inflation rate could vary between 2.37% (minimum) and 10% (maximum) annually, depending on the staking percentage relative to the total coin supply.

2. **Total Coin Supply**: The exact number of coins created per block also depends on the total supply of coins in the network. which is `2,000,000` Let's denote this as \( S \).

3. **Block Time**: The time it takes to produce a block on the network. Polkadot, for example, has a block time of approximately 6 seconds.

4. **Blocks per Year Calculation**: The number of blocks per year is calculated by considering the block time. For a block time of 6 seconds:
   ```
   Blocks per year = (365 * 24 * 3600) / 6 = 5,256,000 blocks per year
   ```

5. **Coins Created per Block**:
   - If the inflation rate is at its maximum (10%), the total new coins created annually would be \( 0.1 \times S \).
   - To find the number of coins created per block:
     ```
     Coins per block = ( 0.1 * S ) / 5,256,000
     ```

### Exact Calculation

Assuming a total coin supply \( S \) of 10 million coins (2,000,000) and an inflation rate at the maximum (2.37%), the calculation would be:
```
Coins per block = ( 2.37 * 2,000,000 ) / 5,256,000 ~ 0.901826484 coins per block
```

The actual number of coins per block will vary based on the real-time staking rate and total supply dynamics. The actual live data from your blockchain will need to fetch current values of the total supply and dynamically calculate the inflation rate based on the current staking percentage.




## Calculation of total payout for the era based on the actual staking rate

### Variables
- \( T_s \) = `npos_token_staked`: Number of tokens that are staked by nominators and validators.
- \( T \) = `total_tokens`: Total number of tokens in circulation.
- \( D \) = `era_duration`: Duration of the era for which the payout is being calculated, in milliseconds.
- \( Y \) = `MILLISECONDS_PER_YEAR`: Number of milliseconds in a Julian year (365.25 days).

### Inflation Function
- `I(t)` = `yearly_inflation`: A piecewise linear function that defines the inflation rate as a function of the staking rate, `t`, where `t = T_s/T` (the fraction of total tokens that are staked).

### Calculation Components
1. **Proportional Time Factor**:
   - The proportional time factor `P` is calculated by the ratio of the era duration to the number of milliseconds in a year, which determines what fraction of the annual inflation should be applied:
   ```
   P = D/Y
   ```

2. **Yearly Inflation from Staking**:
   - The yearly inflation `I(T_s, T)` depends on the staking rate. It is calculated as:
   ```
   I_year = I * (T_s / T) * T
   ```
   This gives the amount of new tokens created per year due to inflation, based on the current staking rate.

3. **Total Payout for the Era**:
   - The total payout for the era ` P_total ` is then:
   ```
   P_total = P * I_year = (D / Y) * I * (T_s / T) * T
   ```

4. **Maximum Inflation Scenario**:
   - If `I_max` is the maximum rate defined in the yearly inflation model, the maximum potential payout for the era is calculated similarly:
   ```
   P_max = P * I_max * T = (D / Y) * I_max * T
   ```

### Final Output
The function returns a tuple of `(P_total`, `P_max)`, which are the total payout and the maximum payout respectively, calculated for the given era duration based on the current and maximum staking inflation scenarios.

### Conclusion
This mathematical representation helps to clarify how the staking proportion and era duration influence the inflation-derived payouts. It shows that the payout is directly proportional to both the era's length relative to a year and the inflationary response to the fraction of tokens staked. This framework ensures that the incentives for staking are aligned with the network's economic security objectives, dynamically adjusting to the staking behavior of the network participants.
