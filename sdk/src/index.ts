import {
  Client as ContractClient,
  Merchant,
  PaymentCharge,
  Refund,
  Dispute,
  PaymentStatus,
  RefundStatus,
  DisputeStatus,
} from "./contracts/fluxapay/src/index.js";
import { Networks } from "@stellar/stellar-sdk";
import {
  FluxapayOfflineSigner,
  OfflineTransactionPayload,
  buildOfflinePayload,
  buildCreatePaymentPayload,
  buildVerifyPaymentPayload,
  buildCreateRefundPayload,
  prepareForOfflineSigning,
  restoreFromOfflinePayload,
} from "./offline-signer.js";
import { NetworkProfileSwitcher, NetworkEnvironment, NetworkProfiles, NetworkProfile } from "./network-profiles.js";

export interface FluxapayConfig {
  network: NetworkEnvironment;
  rpcUrl?: string;
  contractId: string;
}

export class FluxapayClient {
  public contract: ContractClient;
  public networkSwitcher: NetworkProfileSwitcher;

  constructor(config: FluxapayConfig) {
    this.networkSwitcher = new NetworkProfileSwitcher(config.network);
    
    // Override RPC URL if provided, otherwise use the default for the profile
    const rpcUrl = config.rpcUrl || this.networkSwitcher.getProfile().rpcUrl;
    
    this.contract = new ContractClient({
      networkPassphrase: this.networkSwitcher.getProfile().networkPassphrase,
      rpcUrl: rpcUrl,
      contractId: config.contractId,
    });
  }

  /**
   * Switch the client to a different network environment.
   * This re-initializes the contract client seamlessly.
   */
  public switchNetwork(environment: NetworkEnvironment, contractId?: string): void {
    this.networkSwitcher.switchEnvironment(environment);
    const profile = this.networkSwitcher.getProfile();
    const newContractId = contractId || profile.defaultContractId || this.contract.options.contractId;
    
    this.contract = new ContractClient({
      networkPassphrase: profile.networkPassphrase,
      rpcUrl: profile.rpcUrl,
      contractId: newContractId,
    });
  }

  /**
   * Create a new payment charge
   */
  async createPayment(params: {
    paymentId: string;
    merchantId: string;
    amount: bigint;
    currency: string;
    depositAddress: string;
    expiresAt: bigint;
  }) {
    return this.contract.create_payment({
      payment_id: params.paymentId,
      merchant_id: params.merchantId,
      amount: params.amount,
      currency: params.currency,
      deposit_address: params.depositAddress,
      expires_at: params.expiresAt,
    });
  }

  /**
   * Verify a payment via oracle
   */
  async verifyPayment(params: {
    oracle: string;
    paymentId: string;
    transactionHash: Buffer;
    payerAddress: string;
    amountReceived: bigint;
  }) {
    return this.contract.verify_payment({
      oracle: params.oracle,
      payment_id: params.paymentId,
      transaction_hash: params.transactionHash,
      payer_address: params.payerAddress,
      amount_received: params.amountReceived,
    });
  }

  /**
   * Create a refund request
   */
  async createRefund(params: {
    paymentId: string;
    amount: bigint;
    reason: string;
    requester: string;
  }) {
    return this.contract.create_refund({
      payment_id: params.paymentId,
      refund_amount: params.amount,
      reason: params.reason,
      requester: params.requester,
    });
  }

  /**
   * Get merchant details
   */
  async getMerchant(merchantId: string) {
    return this.contract.get_merchant({
      merchant_id: merchantId,
    });
  }

  /**
   * Get payment details
   */
  async getPayment(paymentId: string) {
    return this.contract.get_payment({ payment_id: paymentId });
  }

  /** Offline/hardware wallet payload builder utilities. */
  offlineSigner(): FluxapayOfflineSigner {
    return new FluxapayOfflineSigner(
      this.contract as import("./offline-signer.js").OfflineCapableClient,
      this.contract.options.contractId,
      this.contract.options.networkPassphrase,
    );
  }
}

export {
  Merchant,
  PaymentCharge,
  Refund,
  Dispute,
  PaymentStatus,
  RefundStatus,
  DisputeStatus,
  FluxapayOfflineSigner,
  OfflineTransactionPayload,
  buildOfflinePayload,
  buildCreatePaymentPayload,
  buildVerifyPaymentPayload,
  buildCreateRefundPayload,
  prepareForOfflineSigning,
  restoreFromOfflinePayload,
  NetworkProfileSwitcher,
  NetworkEnvironment,
  NetworkProfiles,
  NetworkProfile,
};
