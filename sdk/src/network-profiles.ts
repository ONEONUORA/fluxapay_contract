import { Networks } from "@stellar/stellar-sdk";

export type NetworkEnvironment = "mainnet" | "testnet" | "standalone";

export interface NetworkProfile {
  environment: NetworkEnvironment;
  networkPassphrase: string;
  rpcUrl: string;
  defaultContractId?: string;
}

export const NetworkProfiles: Record<NetworkEnvironment, NetworkProfile> = {
  mainnet: {
    environment: "mainnet",
    networkPassphrase: Networks.PUBLIC,
    rpcUrl: "https://soroban-rpc.stellar.org",
  },
  testnet: {
    environment: "testnet",
    networkPassphrase: Networks.TESTNET,
    rpcUrl: "https://soroban-testnet.stellar.org",
  },
  standalone: {
    environment: "standalone",
    networkPassphrase: Networks.STANDALONE,
    rpcUrl: "http://localhost:8000/soroban/rpc",
  },
};

export class NetworkProfileSwitcher {
  private currentProfile: NetworkProfile;

  constructor(initialEnvironment: NetworkEnvironment = "testnet") {
    this.currentProfile = NetworkProfiles[initialEnvironment];
  }

  /**
   * Switch the current network environment.
   */
  public switchEnvironment(environment: NetworkEnvironment): void {
    if (!NetworkProfiles[environment]) {
      throw new Error(`Unsupported network environment: ${environment}`);
    }
    this.currentProfile = NetworkProfiles[environment];
  }

  /**
   * Get the active network profile.
   */
  public getProfile(): NetworkProfile {
    return this.currentProfile;
  }

  /**
   * Update the default contract ID for a specific environment.
   */
  public setContractId(environment: NetworkEnvironment, contractId: string): void {
    if (NetworkProfiles[environment]) {
      NetworkProfiles[environment].defaultContractId = contractId;
    }
  }
}
