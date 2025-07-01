# Security Upgrade: Configurable Backend Authorization

## Summary

This upgrade addresses a critical security vulnerability where the backend account for IPFS operations was hardcoded in the pallet. The hardcoded account (`7c650b5b9f657ddcc7a6ddbf9147d33f3b6ffda5009658b1ee6b7e3665a99701`) has been replaced with a configurable governance mechanism.

## Changes Made

### 1. New Configuration Parameter

Added `BackendOrigin` type to the pallet configuration:

```rust
pub trait Config: frame_system::Config + pallet_evm::Config {
    // ... existing config
    /// Origin that can manage authorized backend accounts
    type BackendOrigin: frame_support::traits::EnsureOrigin<Self::RuntimeOrigin>;
}
```

### 2. New Storage Items

- `AuthorizedBackends<T>`: Stores which accounts are authorized to sign IPFS operations

### 3. New Extrinsics

- `authorize_backend`: Allows authorized origin to add backend accounts
- `deauthorize_backend`: Allows authorized origin to remove backend accounts

### 4. Enhanced Security

- The `include_ipfs_hash` function now:
  - Accepts `backend_account` as a parameter
  - Checks if the account is authorized before proceeding
  - Maintains same signature verification logic

## Runtime Configuration Example

```rust
// In your runtime configuration
impl pallet_counter::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SubstrateCurrency = Balances;
    type EvmCurrency = Balances;
    // Use root for backend management (can be changed to council/democracy)
    type BackendOrigin = frame_system::EnsureRoot<AccountId>;
}
```

## Migration Steps

### 1. Update Runtime Configuration

Add the new `BackendOrigin` type to your runtime's pallet configuration.

### 2. Runtime Upgrade

After the runtime upgrade, the existing hardcoded backend account will no longer work. You must authorize backend accounts using the new governance mechanism.

### 3. Authorize Initial Backend

```rust
// Call this after runtime upgrade to authorize your backend account
pallet_counter::authorize_backend(
    origin: Root, // or your configured BackendOrigin
    backend_account: YOUR_BACKEND_ACCOUNT
)
```

### 4. Update Frontend/Backend

Update any frontend or backend code that calls `include_ipfs_hash` to include the `backend_account` parameter.

**Before:**
```rust
include_ipfs_hash(origin, ipfs_hash, backend_signature)
```

**After:**
```rust
include_ipfs_hash(origin, backend_account, ipfs_hash, backend_signature)
```

## Security Benefits

1. **No Single Point of Failure**: Multiple backend accounts can be authorized
2. **Governance Control**: Backend accounts are managed through configurable governance
3. **Transparency**: All backend authorizations are recorded on-chain with events
4. **Flexibility**: Backend accounts can be rotated or revoked as needed
5. **Audit Trail**: All backend operations can be traced to specific authorized accounts

## Alternative BackendOrigin Configurations

### Using Council
```rust
type BackendOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
```

### Using Democracy
```rust
type BackendOrigin = pallet_democracy::EnsureOrigin<RuntimeOrigin>;
```

### Using Custom Origin
```rust
type BackendOrigin = EnsureSignedBy<BackendManagers, AccountId>;
```

## Events

The pallet now emits these additional events:
- `BackendAuthorized(AccountId)`: When a backend account is authorized
- `BackendDeauthorized(AccountId)`: When a backend account is deauthorized

## Breaking Changes

⚠️ **BREAKING CHANGE**: The `include_ipfs_hash` extrinsic signature has changed. All frontend/backend integrations must be updated.

## Testing

After deployment, verify the fix by:

1. Confirming old hardcoded account no longer works
2. Authorizing a new backend account via governance
3. Testing IPFS hash inclusion with the new account
4. Testing deauthorization functionality