# Milestone Flow Analysis & Test Coverage

## Overview of the Milestone Flow

### 1. **Start Milestone** (`initialiseMilestone`)
**Purpose**: Begin a new milestone cycle for airdrop distribution

**What happens**:
- Sets `milestone_active = true`
- Resets `milestone_backers_weighted = 0`
- Does NOT reset `milestone_units_assigned` (stays at 0 from previous milestone end)
- Increments `current_airdrop_cycle` (happens at end of previous milestone)

**State changes**:
```rust
proposal.milestone_active = true
proposal.milestone_backers_weighted = 0
```

**Key invariants**:
- Requires: `is_pool_launched = true`
- Requires: `milestone_active = false` (can't start if already active)
- Requires: `!is_rejected`

---

### 2. **Snapshot Backer** (`snapshotBackerAmount`)
**Purpose**: Calculate and assign milestone rewards to a backer based on their token holdings

**What happens**:
1. Calculates expected vs actual token holdings
2. Determines tier percentage (0%, 25%, 50%, 75%, 100%) based on holding percentage
3. Calculates `alloc_units` = `(per_backer_amount * tier_pct) / 100`
4. **Increments** `backer_account.claim_amount += alloc_units`
5. **Increments** `proposal.milestone_units_assigned += alloc_units` ⚠️ **CRITICAL**
6. Updates `backer_account.settle_cycle = current_airdrop_cycle`
7. Increments `proposal.milestone_backers_weighted += 1`

**State changes**:
```rust
backer_account.claim_amount += alloc_units  // Base units
proposal.milestone_units_assigned += alloc_units  // Base units
backer_account.settle_cycle = current_airdrop_cycle
proposal.milestone_backers_weighted += 1
```

**Key invariants**:
- Requires: `milestone_active = true`
- Requires: `current_airdrop_cycle > backer_account.settle_cycle`
- Must be called for ALL backers before ending milestone

**Tier calculation**:
- 100%: Holding ≥ 100% of expected
- 75%: Holding ≥ 70% of expected
- 50%: Holding ≥ 50% of expected
- 25%: Holding ≥ 25% of expected
- 0%: Holding < 25% of expected

---

### 3. **Claim Tokens** (`claimMilestoneReward`)
**Purpose**: Transfer assigned tokens from vault to backer

**What happens**:
1. Reads `backer_account.claim_amount` (in base units)
2. Converts to raw units: `claim_amount * 10^decimals`
3. Transfers tokens from `token_vault` to `backer_token_account`
4. **Decrements** `proposal.milestone_units_assigned -= claim_amount` ⚠️ **CRITICAL FIX**
5. Resets `backer_account.claim_amount = 0`
6. Updates `backer_account.claimed_upto = settle_cycle`

**State changes**:
```rust
proposal.milestone_units_assigned -= claim_amount_base  // Decrement unclaimed tracker
backer_account.claim_amount = 0
backer_account.claimed_upto = backer_account.settle_cycle
token_vault.amount -= claim_amount_raw
backer_token_account.amount += claim_amount_raw
```

**Key invariants**:
- Requires: `is_pool_launched = true`
- Can be called multiple times (if claim_amount > 0)
- Can be called BEFORE or AFTER ending milestone (but tokens must be assigned first)

---

### 4. **End Milestone** (`endMilestone`)
**Purpose**: Burn unclaimed tokens and prepare for next milestone

**What happens**:
1. Calculates burn amount: `B = A_total - A_unclaimed`
   - `A_total = config.total_airdrop_amount_per_milestone * 10^decimals` (raw units)
   - `A_unclaimed = proposal.milestone_units_assigned * 10^decimals` (raw units)
2. Burns `burn_amount` tokens from `token_vault`
3. Sets `milestone_active = false`
4. Resets `milestone_units_assigned = 0` (for next milestone)
5. Increments `current_airdrop_cycle += 1`

**State changes**:
```rust
token_vault.amount -= burn_amount  // Burn unclaimed tokens
proposal.milestone_active = false
proposal.milestone_units_assigned = 0  // Reset for next milestone
proposal.current_airdrop_cycle += 1
```

**Key invariants**:
- Requires: `milestone_active = true`
- Requires: `milestone_backers_weighted == total_backers` (all backers snapshotted)
- Requires: `is_pool_launched = true`
- Requires: `!is_rejected`

**Burn Formula**:
```
burn_amount = (total_airdrop_allocation - unclaimed_tokens) in raw token units
```

---

## Flow Diagram

```
┌─────────────────┐
│ Start Milestone │
│ milestone_active│
│   = true        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Snapshot Backer │  (repeat for each backer)
│ milestone_units │
│ _assigned += X  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Claim Tokens   │  (can happen multiple times, anytime)
│ milestone_units│
│ _assigned -= X  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  End Milestone  │
│ Burn unclaimed  │
│ Reset counters  │
└─────────────────┘
```

---

## Critical Tracking Logic

### `milestone_units_assigned` Tracking

**Before Fix**:
- ✅ Incremented in snapshot: `+= alloc_units`
- ❌ NOT decremented in claim (BUG!)
- Result: Tracked total assigned, NOT unclaimed

**After Fix**:
- ✅ Incremented in snapshot: `+= alloc_units`
- ✅ Decremented in claim: `-= claim_amount`
- Result: Always tracks unclaimed tokens correctly

**Example Flow**:
```
Start milestone: milestone_units_assigned = 0

Snapshot backer 1: alloc_units = 100
  → milestone_units_assigned = 100

Snapshot backer 2: alloc_units = 50
  → milestone_units_assigned = 150

Claim backer 1: claim_amount = 100
  → milestone_units_assigned = 50  ✅ (unclaimed)

Claim backer 2: claim_amount = 50
  → milestone_units_assigned = 0   ✅ (all claimed)

End milestone: burn_amount = total_allocation - 0 = total_allocation
```

---

## Test Coverage Analysis

### ✅ What Tests Cover

1. **Basic Flow** (Tests 12-15):
   - ✅ Start milestone
   - ✅ Snapshot backer
   - ✅ End milestone
   - ✅ Claim tokens

2. **Error Cases**:
   - ✅ Unauthorized access
   - ✅ Invalid milestone state
   - ✅ Missing snapshot operations

3. **Token Transfer**:
   - ✅ Verifies backer receives tokens
   - ✅ Verifies claim_amount resets to 0

### ❌ Critical Test Gaps

1. **Burn Calculation Verification**:
   - ❌ No test verifies vault balance before/after burn
   - ❌ No test verifies burn_amount calculation
   - ❌ No test verifies unclaimed tokens are burned correctly

2. **milestone_units_assigned Tracking**:
   - ❌ No test verifies `milestone_units_assigned` increments in snapshot
   - ❌ No test verifies `milestone_units_assigned` decrements in claim
   - ❌ No test verifies final value matches unclaimed tokens

3. **Order of Operations**:
   - ⚠️ Test 15 claims AFTER ending milestone (should work but not ideal)
   - ❌ No test for: snapshot → claim → end milestone (proper order)
   - ❌ No test for: snapshot → end milestone → claim (current order)

4. **Partial Claims**:
   - ❌ No test for multiple claims before ending milestone
   - ❌ No test for some backers claiming, others not

5. **Burn Edge Cases**:
   - ❌ No test for burn when all tokens claimed (should burn 0)
   - ❌ No test for burn when no tokens claimed (should burn all)
   - ❌ No test for burn when partial tokens claimed

---

## Recommended Test Additions

### Test 1: Verify milestone_units_assigned Tracking
```typescript
it('Verifies milestone_units_assigned tracks unclaimed tokens correctly', async () => {
  // Start milestone
  await program.methods.initialiseMilestone()...
  
  // Snapshot backer
  const proposalBefore = await program.account.proposal.fetch(proposal);
  assert.strictEqual(proposalBefore.milestoneUnitsAssigned.toNumber(), 0);
  
  await program.methods.snapshotBackerAmount()...
  
  const proposalAfterSnapshot = await program.account.proposal.fetch(proposal);
  const assignedAfterSnapshot = proposalAfterSnapshot.milestoneUnitsAssigned.toNumber();
  assert.ok(assignedAfterSnapshot > 0, "Should increment on snapshot");
  
  // Claim tokens
  await program.methods.claimMilestoneReward()...
  
  const proposalAfterClaim = await program.account.proposal.fetch(proposal);
  const assignedAfterClaim = proposalAfterClaim.milestoneUnitsAssigned.toNumber();
  assert.strictEqual(assignedAfterClaim, assignedAfterSnapshot - expectedClaimAmount, 
    "Should decrement by claim amount");
});
```

### Test 2: Verify Burn Calculation
```typescript
it('Verifies burn calculation burns unclaimed tokens correctly', async () => {
  // Setup: snapshot, claim some, don't claim others
  
  const vaultBefore = await provider.connection.getTokenAccountBalance(vault);
  const totalAllocation = config.totalAirdropAmountPerMilestone * Math.pow(10, 9);
  
  await program.methods.endMilestone()...
  
  const vaultAfter = await provider.connection.getTokenAccountBalance(vault);
  const burned = vaultBefore.value.amount - vaultAfter.value.amount;
  
  const expectedUnclaimed = /* calculate from milestone_units_assigned */;
  const expectedBurned = totalAllocation - expectedUnclaimed;
  
  assert.strictEqual(burned, expectedBurned, "Burn amount should match calculation");
});
```

### Test 3: Verify Complete Flow with Multiple Backers
```typescript
it('Verifies complete flow: snapshot → claim → end milestone', async () => {
  // Start milestone
  // Snapshot backer 1
  // Snapshot backer 2
  // Claim backer 1
  // End milestone (should burn backer 2's unclaimed tokens)
  // Verify burn amount = backer 2's allocation
});
```

---

## Summary

### Flow Correctness: ✅ Fixed
- The burn calculation now correctly uses `milestone_units_assigned` as unclaimed tokens
- Tracking is maintained correctly through snapshot (increment) and claim (decrement)

### Test Coverage: ⚠️ Needs Improvement
- Basic flow is tested but lacks verification of:
  - Burn calculation correctness
  - `milestone_units_assigned` tracking
  - Vault balance changes
  - Edge cases (all claimed, none claimed, partial)

### Critical Missing Tests
1. Burn amount verification
2. `milestone_units_assigned` increment/decrement tracking
3. Vault balance before/after burn
4. Multiple claim scenarios
5. Edge cases for burn calculation

