# Admin Security & Safety Guide

## ✅ Using Squads Multi-Sig Wallet (Recommended)

**Yes, you can and should set the admin account to a Squads wallet!** This is a best practice for production deployments.

### How to Set Admin to Squads Wallet

1. **Get your Squads wallet address** (the multi-sig wallet address)
2. **Update `constant.rs`**:
   ```rust
   pub mod admin_pubkey {
       use anchor_lang::{prelude::Pubkey, solana_program::pubkey};
       pub const ID: Pubkey = pubkey!("YOUR_SQUADS_WALLET_ADDRESS_HERE");
   }
   ```
3. **Rebuild and redeploy**: `anchor build && anchor deploy`

### Why Squads is Better

- ✅ **Multi-signature protection**: Requires multiple approvals
- ✅ **Key management**: No single point of failure
- ✅ **Audit trail**: All transactions require multiple signatures
- ✅ **Recovery options**: Can rotate signers without changing admin address
- ✅ **Governance**: Better aligns with company leadership structure

## 🔐 Admin Powers & Functions

The admin account has access to these functions:

### 1. **setConfig** ⚠️ HIGH IMPACT
- **What it does**: Sets global program configuration
- **Parameters**: 
  - `amount_to_raise_per_user` - Max SOL per user
  - `total_mint` - Total token supply
  - `total_pool_tokens` - Pool token allocation
  - `maker_token_amount` - Maker token allocation
  - `total_airdrop_amount_per_milestone` - Airdrop amounts
  - `min_backers` - Minimum backers required
  - `max_backed_proposals` - Max proposals per backer
  - `refund_fee_basis_points` - Refund fee percentage
- **Risk Level**: 🟡 **MEDIUM** - Affects all future proposals
- **Safety Net**: Only affects new proposals, not existing ones

### 2. **rejectProposal** ⚠️ HIGH IMPACT
- **What it does**: Rejects a proposal, allowing backers to refund
- **Risk Level**: 🟡 **MEDIUM** - Can disrupt active proposals
- **Safety Net**: Backers can still claim refunds

### 3. **snapshotBackerAmount** ⚠️ MEDIUM IMPACT
- **What it does**: Snapshots backer amounts for milestone calculations
- **Risk Level**: 🟢 **LOW** - Read-only operation, doesn't change state
- **Safety Net**: Can be called multiple times safely

### 4. **initialiseMilestone** ⚠️ HIGH IMPACT
- **What it does**: Starts a new milestone cycle for a proposal
- **Risk Level**: 🟡 **MEDIUM** - Triggers milestone distribution
- **Safety Net**: Must be called in sequence, can't skip milestones

### 5. **endMilestone** ⚠️ HIGH IMPACT
- **What it does**: Ends current milestone, burns unused tokens
- **Risk Level**: 🟡 **MEDIUM** - Permanently burns tokens
- **Safety Net**: Only burns tokens that weren't claimed

### 6. **emergencyUnlock** ⚠️ CRITICAL (Currently Disabled)
- **What it does**: Emergency unlock if pool creation fails
- **Status**: ❌ **DISABLED** (commented out in `lib.rs`)
- **Risk Level**: 🔴 **HIGH** - Could unlock funds incorrectly
- **Safety Nets** (if enabled):
  - ✅ Requires pool doesn't exist OR 24+ hours passed
  - ✅ Can only be used once per proposal
  - ✅ Requires pool launch flag to be set
- **Recommendation**: Keep disabled unless absolutely necessary

## 🛡️ Safety Nets & Protections

### Built-in Safety Features

1. **Single Admin Check**
   - ✅ Only one admin address can execute admin functions
   - ✅ Simple key comparison prevents unauthorized access

2. **No Direct Fund Control**
   - ✅ Admin cannot directly withdraw funds
   - ✅ Funds are held in PDAs controlled by program
   - ✅ Vault authority is a PDA (no private key)

3. **Config Changes Don't Affect Existing Proposals**
   - ✅ `setConfig` only affects new proposals
   - ✅ Existing proposals use their original config

4. **Emergency Unlock Protections** (if enabled)
   - ✅ 24-hour time lock
   - ✅ One-time use per proposal
   - ✅ Requires pool verification

5. **Program Upgradeability**
   - ✅ Program is upgradeable (from `Anchor.toml`)
   - ⚠️ **RISK**: Admin with program upgrade authority can change code
   - 💡 **Recommendation**: Use separate upgrade authority (not admin)

### Current Limitations & Risks

1. **No Timelock**
   - ❌ Admin actions execute immediately
   - 💡 **Recommendation**: Consider adding timelock for critical operations

2. **No Multi-Sig at Program Level**
   - ❌ Program only checks single admin address
   - ✅ **Solution**: Use Squads wallet (multi-sig at wallet level)

3. **No Governance Token**
   - ❌ No token-based voting for admin actions
   - 💡 **Future Consideration**: Could add governance layer

4. **Program Upgrade Risk**
   - ⚠️ Program deployer can upgrade program
   - ⚠️ Upgrade could change admin logic
   - 💡 **Recommendation**: 
     - Use separate upgrade authority
     - Consider making program immutable after launch
     - Or use multi-sig for upgrade authority

## 📋 Recommended Security Setup

### Production Setup

1. **Admin Authority**: Squads multi-sig wallet (3-of-5 or 4-of-7 recommended)
   - Company leadership as signers
   - Geographic distribution
   - Hardware wallet support

2. **Program Upgrade Authority**: Separate multi-sig (different from admin)
   - More restrictive threshold (e.g., 5-of-7)
   - Separate from daily operations

3. **Treasury Address**: Already fixed at `76U9hvHNUNn7YV5FekSzDHzqnHETsUpDKq4cMj2dMxNi`
   - Verify this is also a multi-sig wallet
   - Separate from admin operations

### Security Checklist

- [ ] Admin set to Squads multi-sig wallet
- [ ] Upgrade authority is separate multi-sig
- [ ] Treasury address is multi-sig wallet
- [ ] All signers use hardware wallets
- [ ] Signers geographically distributed
- [ ] Emergency procedures documented
- [ ] Regular security audits scheduled
- [ ] Access logs monitored

## 🔄 Changing Admin Address

### Process

1. **Update `constant.rs`**:
   ```rust
   pub mod admin_pubkey {
       use anchor_lang::{prelude::Pubkey, solana_program::pubkey};
       pub const ID: Pubkey = pubkey!("NEW_SQUADS_ADDRESS");
   }
   ```

2. **Rebuild program**:
   ```bash
   anchor build
   ```

3. **Deploy upgrade**:
   ```bash
   anchor deploy
   ```

4. **Verify**:
   ```bash
   # Test admin function with new wallet
   node scripts/set-config.js <path-to-squads-keypair>
   ```

### Important Notes

- ⚠️ **Old admin loses access immediately** after upgrade
- ⚠️ **New admin gains access immediately** after upgrade
- ✅ **No downtime** - upgrade is atomic
- ✅ **Existing proposals unaffected** - only new admin operations use new address

## 🚨 Emergency Procedures

### If Admin Key is Compromised

1. **Immediate Actions**:
   - Freeze admin operations (if possible)
   - Prepare program upgrade with new admin
   - Deploy emergency upgrade

2. **Recovery Steps**:
   - Update admin to new secure wallet
   - Deploy program upgrade
   - Verify new admin works
   - Monitor for unauthorized actions

### If Program Needs Emergency Changes

1. **Upgrade Authority** (separate from admin):
   - Can upgrade program code
   - Can fix bugs or add features
   - Should also be multi-sig

2. **Admin Authority**:
   - Can change config
   - Can manage proposals
   - Cannot change program code

## 📊 Risk Assessment

| Function | Risk Level | Impact | Reversibility |
|----------|-----------|--------|---------------|
| `setConfig` | 🟡 Medium | Affects all new proposals | Reversible (can change again) |
| `rejectProposal` | 🟡 Medium | Affects one proposal | Irreversible (but backers can refund) |
| `snapshotBackerAmount` | 🟢 Low | Read-only operation | N/A |
| `initialiseMilestone` | 🟡 Medium | Starts milestone cycle | Can't undo, but can end milestone |
| `endMilestone` | 🟡 Medium | Burns unclaimed tokens | Irreversible |
| `emergencyUnlock` | 🔴 High | Unlocks proposal | Irreversible (currently disabled) |

## 💡 Best Practices

1. **Use Multi-Sig**: Always use Squads or similar for admin
2. **Separate Authorities**: Keep upgrade authority separate from admin
3. **Hardware Wallets**: Use hardware wallets for all signers
4. **Documentation**: Document all admin procedures
5. **Monitoring**: Monitor all admin transactions
6. **Testing**: Test admin functions on devnet first
7. **Backup Plans**: Have recovery procedures ready
8. **Regular Audits**: Schedule regular security audits

## 🔍 Monitoring Admin Actions

### What to Monitor

- All `setConfig` calls (config changes)
- All `rejectProposal` calls (proposal rejections)
- All `initialiseMilestone` / `endMilestone` calls (milestone management)
- Program upgrade transactions
- Unusual transaction patterns

### Tools

- Solana Explorer: Monitor transactions
- Custom monitoring scripts
- Alert systems for admin operations
- Regular review of admin activity logs

## 📝 Summary

**Yes, use Squads wallet for admin!** It's the right approach for production.

**Safety Nets**:
- ✅ Multi-sig protection (via Squads)
- ✅ No direct fund control
- ✅ Config changes don't affect existing proposals
- ✅ Emergency unlock disabled
- ✅ Program upgradeability (but use separate authority)

**Recommendations**:
- Use Squads multi-sig (3-of-5 or 4-of-7)
- Separate upgrade authority from admin
- Use hardware wallets
- Monitor all admin actions
- Keep emergency unlock disabled unless needed

