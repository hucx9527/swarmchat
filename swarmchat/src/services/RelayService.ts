/**
 * RelayService — HTTP client for the SCP relay node.
 */

import { DEFAULT_RELAY_URL } from '../utils/constants';

interface RelayResponse {
  status?: string;
  error?: string;
  [key: string]: any;
}

class RelayServiceClass {
  private relayUrl: string;

  constructor() {
    this.relayUrl = DEFAULT_RELAY_URL;
  }

  setRelayUrl(url: string): void {
    this.relayUrl = url.replace(/\/+$/, '');
  }

  getRelayUrl(): string {
    return this.relayUrl;
  }

  // ---- Health & Info ----

  async health(): Promise<RelayResponse> {
    return this.get('/health');
  }

  async nodeInfo(): Promise<RelayResponse> {
    return this.get('/node/info');
  }

  // ---- Prekey Bundles (§4.6) ----

  async uploadPrekey(
    did: string,
    identityKey: string,
    signedPrekey: string,
    spkSignature: string,
    oneTimePrekeys: string[],
  ): Promise<RelayResponse> {
    return this.post('/prekey', {
      did,
      identity_key: identityKey,
      signed_prekey: {
        key: signedPrekey,
        signature: spkSignature,
      },
      one_time_prekeys: oneTimePrekeys,
    });
  }

  async getPrekey(did: string): Promise<{
    did: string;
    identity_key: string;
    signed_prekey: { key: string; signature: string };
    one_time_prekey: string;
    prekeys_remaining: number;
  }> {
    return this.get(`/prekey/${did}`);
  }

  // ---- Offline Messages (§5.5) ----

  async storeMessage(
    toDid: string,
    fromDid: string,
    envelope: string,
    messageId: string,
    ttl: number = 604800,
  ): Promise<RelayResponse> {
    return this.post('/message', {
      to_did: toDid,
      from_did: fromDid,
      message_id: messageId,
      envelope,
      ttl,
    });
  }

  async syncMessages(
    did: string,
    since: number = 0,
    limit: number = 50,
  ): Promise<{
    messages: Array<{
      id: string;
      from_did: string;
      to_did: string;
      envelope: string;
      timestamp: number;
      ttl: number;
    }>;
    has_more: boolean;
    sync_token?: string;
  }> {
    return this.get(`/sync/${did}?since=${since}&limit=${Math.min(limit, 500)}`);
  }

  // ---- HTTP Helpers ----

  private async get(path: string): Promise<any> {
    const url = `${this.relayUrl}${path}`;
    try {
      const response = await fetch(url, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
          'User-Agent': 'SwarmChat/0.1.0',
        },
      });

      if (!response.ok) {
        const body = await response.text();
        throw new Error(`HTTP ${response.status}: ${body}`);
      }

      return response.json();
    } catch (error: any) {
      if (error.message?.startsWith('HTTP')) throw error;
      throw new Error(`Relay unreachable: ${error.message}`);
    }
  }

  private async post(path: string, body: any): Promise<any> {
    const url = `${this.relayUrl}${path}`;
    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json',
          'User-Agent': 'SwarmChat/0.1.0',
        },
        body: JSON.stringify(body),
      });

      if (!response.ok) {
        const text = await response.text();
        throw new Error(`HTTP ${response.status}: ${text}`);
      }

      return response.json();
    } catch (error: any) {
      if (error.message?.startsWith('HTTP')) throw error;
      throw new Error(`Relay unreachable: ${error.message}`);
    }
  }
}

export const RelayService = new RelayServiceClass();
export default RelayService;
