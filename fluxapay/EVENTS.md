# Fluxapay Contract Events

This document defines the on-chain events emitted by the Fluxapay smart contracts. These events are used by the backend and oracle systems to track state changes and trigger off-chain processes.

## Merchant Events

Emitted by the `MerchantRegistry` contract.

### MERCHANT/REGISTERED
Emitted when a new merchant registers on the platform.
- **Topics**: `(MERCHANT, REGISTERED)`
- **Data**: `(merchant_id: Address, settlement_currency: String)`

### MERCHANT/VERIFIED
Emitted when a merchant's KYC status is verified by an admin.
- **Topics**: `(MERCHANT, VERIFIED)`
- **Data**: `merchant_id: Address`

### MERCHANT/UPDATED
Emitted when a merchant's profile or configuration is updated.
- **Topics**: `(MERCHANT, UPDATED)`
- **Data**: `merchant_id: Address`

## Payment Events

Emitted by the `PaymentProcessor` contract.

### PAYMENT/CREATED
Emitted when a new payment charge is created.
- **Topics**: `(PAYMENT, CREATED)`
- **Data**: `payment_id: String`

### PAYMENT/VERIFIED
Emitted when a payment is successfully confirmed on-chain.
- **Topics**: `(PAYMENT, VERIFIED)`
- **Data**: `payment_id: String`

### PAYMENT/FAILED
Emitted when a payment verification fails (e.g., incorrect amount received).
- **Topics**: `(PAYMENT, FAILED)`
- **Data**: `payment_id: String`

## Refund Events

Emitted by the `RefundManager` contract.

### REFUND/CREATED
Emitted when a refund request is initiated.
- **Topics**: `(REFUND, CREATED)`
- **Data**: `(payment_id: String, refund_id: String, refund_amount: i128)`

### REFUND/COMPLETED
Emitted when a refund is successfully processed and funds are sent.
- **Topics**: `(REFUND, COMPLETED)`
- **Data**: `(payment_id: String, refund_id: String, refund_amount: i128)`

### REFUND/REJECTED
Emitted when a refund request is rejected by an operator.
- **Topics**: `(REFUND, REJECTED)`
- **Data**: `(payment_id: String, refund_id: String, refund_amount: i128)`

## Dispute Events

Emitted by the `RefundManager` contract.

### DISPUTE/OPENED
Emitted when a new dispute is opened for a payment.
- **Topics**: `(DISPUTE, OPENED)`
- **Data**: `(payment_id: String, dispute_id: String, amount: i128)`

### DISPUTE/UNDER_REVIEW
Emitted when a dispute's status is changed to under review.
- **Topics**: `(DISPUTE, UNDER_REVIEW)`
- **Data**: `(payment_id: String, dispute_id: String, amount: i128)`

### DISPUTE/RESOLVED
Emitted when a dispute is resolved in favor of the customer (refund issued).
- **Topics**: `(DISPUTE, RESOLVED)`
- **Data**: `(payment_id: String, dispute_id: String, amount: i128)`

### DISPUTE/REJECTED
Emitted when a dispute is rejected by an operator.
- **Topics**: `(DISPUTE, REJECTED)`
- **Data**: `(payment_id: String, dispute_id: String, amount: i128)`
