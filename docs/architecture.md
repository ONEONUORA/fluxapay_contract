# FluxaPay Contract Architecture

## Overview

FluxaPay is a multi-contract Soroban dApp that enables merchants to accept payments, create payment links, manage refunds, and handle disputes on the Stellar network. The system is built around three core contracts that coordinate payment processing, refund management, and merchant registry.

---

## Contract Responsibilities

### PaymentProcessor

The **PaymentProcessor** contract is the primary contract that orchestrates payment creation, settlement, and lifecycle management. It maintains:
- Payment records with status tracking (PENDING, SETTLED, DISPUTED, REFUNDED)
- Rate limiting per merchant and payer to prevent abuse
- Idempotency keys to prevent duplicate payment creation
- Fee configuration and split logic (treasury, developer, merchant)
- Integration with the FX oracle for multi-currency settlements
- Payment link management via PaymentLinkManager
- Dispute tracking and arbitration voting
- Subscription plan and tick-based recurring payments

### RefundManager

The **RefundManager** contract handles all refund operations, including:
- Refund creation and status tracking
- Cooldown periods to prevent rapid refund abuse
- Collaborative settlement signatures for operator/merchant agreement
- Refund routing (direct to payer or via DEX swap to original token)
- Reentrancy protection during concurrent refund processing

### MerchantRegistry

The **MerchantRegistry** contract maintains merchant data and verification:
- Merchant registration with KYC tier levels
- Merchant verification and status management
- Monthly and cumulative volume tracking for tier auto-upgrades
- Dispute counts for merchant reputation scoring
- Merchant-specific rate limiting configuration

---

## Cross-Contract Call Diagram

```
PaymentProcessor
  ├─ calls MerchantRegistry
  │   ├─ verify_merchant(merchant_id) → ok/error
  │   └─ get_merchant(merchant_id) → Merchant
  │
  ├─ calls DexRouter (for swap_and_pay)
  │   └─ swap_exact_tokens_for_tokens() → Vec<amounts>
  │
  ├─ calls FXOracle (optional, for rate validation)
  │   └─ get_rate(pair) → (rate, timestamp)
  │
  ├─ calls RefundManager
  │   ├─ process_refund(refund_id) → ok/error
  │   └─ create_refund(...) → refund_id
  │
  └─ calls PaymentLinkManager
      ├─ create_link(...) → link_id
      └─ use_link(link_id) → payment_id

RefundManager
  ├─ calls DexRouter (for swap-based refunds)
  │   └─ swap_exact_tokens_for_tokens() → amounts
  │
  └─ calls FXOracle (optional, for refund rate validation)
      └─ get_rate(pair) → (rate, timestamp)

PaymentLinkManager
  └─ stores links independently (no external calls)
```

### Key Call Flows

**Payment Creation Flow:**
1. Payer calls `PaymentProcessor::create_payment()`
2. PaymentProcessor validates payer rate limit
3. PaymentProcessor fetches merchant from MerchantRegistry
4. PaymentProcessor stores payment with PENDING status
5. Payment event is emitted
6. Return payment_id to payer

**Payment Settlement Flow:**
1. Operator calls `PaymentProcessor::settle_payment(payment_id)`
2. PaymentProcessor acquires reentrancy lock
3. PaymentProcessor updates payment status to SETTLED
4. Fees are calculated and split
5. Settlement event is emitted
6. Reentrancy lock is released

**Refund Processing Flow:**
1. Operator/Payer calls `RefundManager::process_refund(refund_id)`
2. RefundManager validates cooldown period
3. If swap-based refund: calls DexRouter to exchange tokens
4. Refund status updated to PROCESSED
5. Refund event emitted
6. Funds transferred to original payer

---

## Role Model

### Admin Role

**Permissions:**
- Grant/revoke roles to other accounts
- Initialize contract and set settlement operator
- Claim admin from pending admin (two-step admin transfer)
- Set MerchantRegistry address
- Allow/blacklist tokens
- Set global rate limits and fee configuration
- Propose and finalize fee changes (7-day maturity)
- Set KYC tier limits

**How Granted:** Set at initialization; transferred via `claim_admin()` two-step process

### Merchant Role

**Permissions:**
- Create payment links via PaymentLinkManager
- Register in MerchantRegistry
- Set merchant-specific rate limits
- View their own payments and refunds
- Receive payment settlements

**How Granted:** Admin grants via `grant_role()` after merchant verification

### Oracle Role

**Permissions:**
- Provide FX rate data to PaymentProcessor
- Update current rate and timestamp
- Used to validate swap prices during DEX execution

**How Granted:** Admin grants via `grant_role()` to FXOracle contract or operator

### Settlement Operator Role

**Permissions:**
- Call `settle_payment()` to mark payments as settled
- Call `process_refund()` to execute refunds
- View payment and refund records
- Sign collaborative settlements with merchants
- Propose and finalize fee changes (7-day maturity)

**How Granted:** Admin sets via `set_settlement_operator()` at init; revoked via `revoke_role()`

### Arbitrator Role

**Permissions:**
- Vote on disputes via `vote_on_dispute()`
- Lock stake for dispute arbitration
- View dispute details and voting tally

**How Granted:** Admin grants via `grant_role()` when adding to arbitrator pool

---

## Key DataKey Variants

| DataKey | Type | Purpose |
|---------|------|---------|
| `Payment(id)` | PaymentCharge | Core payment record with amount, merchant, status, fees |
| `Refund(id)` | RefundRecord | Refund request with original token, swap path, status |
| `Dispute(id)` | Dispute | Dispute record with evidence, resolution, voting state |
| `Stream(id)` | PaymentStream | Recurring payment plan with tick interval and balance |
| `MerchantPayments(addr)` | Vec<String> | Index of payment IDs for a merchant (pagination) |
| `MerchantRateLimit(addr)` | (count, reset_time) | Per-merchant creation rate limit tracker |
| `PayerRateLimit(addr)` | (count, reset_time) | Per-payer creation rate limit tracker |
| `GlobalRateLimit` | (count, reset_time) | Global creation rate limit across all users |
| `MerchantMonthlyVolume(addr, month)` | i128 | Cumulative payment volume in current month (for KYC limits) |
| `MerchantCumulativeVolume(addr)` | i128 | All-time volume for tier auto-upgrade eligibility |
| `IdempotencyKey(key)` | String | Stores payment_id to prevent duplicate creation |
| `AllowedToken(addr)` | bool | Whitelist of supported payment tokens |
| `FeeSplitConfig` | FeeSplit | Treasury %, developer %, and addresses for fee distribution |
| `CurrentFee` | i128 | Current payment processing fee in basis points |
| `KycTierLimitsConfig` | Map<tier, limit> | Max monthly volume per KYC tier |
| `DisputeArbitratorVotes(id)` | Vec<Address> | List of arbitrators who voted on a dispute |
| `DisputeVoteTally(id)` | (for_count, against_count) | Vote counts for dispute resolution |
| `ReentrancyLock` | bool | Prevents concurrent settle_payment/process_refund calls |

---

## Payment Lifecycle

### 1. **Creation** (Payer initiates)
   - Payer calls `PaymentProcessor::create_payment()` with merchant and amount
   - PaymentProcessor checks rate limits, validates merchant, creates Payment with status=PENDING
   - Event: `PAYMENT/CREATED(payment_id, payer, merchant, amount)`

### 2. **Settlement** (Operator confirms)
   - Operator calls `PaymentProcessor::settle_payment(payment_id)`
   - PaymentProcessor acquires reentrancy lock, updates status to SETTLED, calculates and splits fees
   - Operator receives settlement confirmation
   - Event: `PAYMENT/SETTLED(payment_id, amount, fee)`

### 3. **Dispute** (Payer/Merchant challenges)
   - Payer calls `PaymentProcessor::create_dispute(payment_id, evidence_hash)` within dispute window
   - PaymentProcessor creates Dispute, emits event, starts arbitration
   - Arbitrators lock stake and vote
   - Event: `DISPUTE/CREATED(dispute_id, payment_id, initiator)`

### 4a. **Refund (Approval)** (Dispute resolved in favor of refund)
   - After dispute resolution or within cooldown period, `RefundManager::process_refund()` is called
   - RefundManager checks cooldown, routes refund (direct or via DEX swap to original token), transfers funds
   - Refund status changes to PROCESSED, original payment status to REFUNDED
   - Event: `REFUND/PROCESSED(refund_id, original_amount, routed_amount)`

### 4b. **Finalization** (Dispute resolved, payment confirmed)
   - Arbitrators vote settlement, tally exceeds threshold
   - Dispute status changes to RESOLVED
   - Payment status remains SETTLED or transitions to DISPUTED then back to SETTLED if arbitration favors payment
   - Event: `DISPUTE/RESOLVED(dispute_id, outcome, final_status)`

### Optional: **Recurring Payments** (Subscription tick)
   - Recurring payments tick at interval; PaymentProcessor calls `process_due_subscriptions()`
   - For each active subscription past its tick time, creates a new payment (child of subscription)
   - Updates subscription balance and next tick time
   - Event: `SUBSCRIPTION/TICK(subscription_id, new_payment_id, remaining_balance)`

---

## Integration Points

- **DEX Router**: Atomic token swaps for `swap_and_pay()` and swap-based refunds
- **FX Oracle**: Optional multi-currency rate validation to prevent price slippage abuse
- **Merchant Registry**: Verification, KYC tier tracking, merchant lookup
- **Payment Link Manager**: Independent links with direct transfers and metadata validation

---

## Security Considerations

- **Reentrancy Protection**: `ReentrancyLock` guards concurrent settle/refund operations
- **Rate Limiting**: Per-merchant, per-payer, and global limits prevent spam
- **Idempotency**: Duplicate payment requests detected via idempotency keys
- **Cooldown Periods**: Refund requests blocked within cooldown window (default 7 days)
- **Collaborative Settlement**: Operator and merchant must both sign refund agreements
- **Arbitration Voting**: Stake-locked voting with threshold ensures fair dispute resolution
- **Two-Step Admin Transfer**: Prevents accidental admin loss via `pending_admin` + `claim_admin()`
- **Metadata Validation**: Payment links enforce max key count (20) and value length (256 chars)
