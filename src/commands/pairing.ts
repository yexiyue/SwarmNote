import { invoke } from "@tauri-apps/api/core";

export interface PairingCodeInfo {
  code: string;
  createdAt: string;
  expiresAt: string;
}

export interface PairedDeviceInfo {
  peerId: string;
  hostname: string;
  os: string;
  platform: string;
  arch: string;
  pairedAt: string;
  lastSeen: string | null;
  isOnline?: boolean;
  rttMs?: number;
}

export interface PeerInfo {
  peer_id: string;
  hostname: string;
  os: string;
  platform: string;
  arch: string;
  is_connected: boolean;
  rtt_ms: number | null;
  connection_type: string | null;
}

export interface ShareCodeDeviceInfo {
  peerId: string;
  osInfo: {
    hostname: string;
    os: string;
    platform: string;
    arch: string;
  };
}

export type PairingMethod = { type: "Direct" } | { type: "Code"; code: string };

export interface PairingResponse {
  status: "Success" | "Refused";
  reason?: string;
}

export async function generatePairingCode(expiresInSecs?: number): Promise<PairingCodeInfo> {
  return invoke("generate_pairing_code", { expiresInSecs });
}

export async function getDeviceByCode(code: string): Promise<ShareCodeDeviceInfo> {
  return invoke("get_device_by_code", { code });
}

export async function requestPairing(
  peerId: string,
  method: PairingMethod,
): Promise<PairingResponse> {
  return invoke("request_pairing", { peerId, method });
}

export async function respondPairingRequest(pendingId: number, accept: boolean): Promise<void> {
  return invoke("respond_pairing_request", { pendingId, accept });
}

export async function getPairedDevices(): Promise<PairedDeviceInfo[]> {
  return invoke("get_paired_devices");
}

export async function unpairDevice(peerId: string): Promise<void> {
  return invoke("unpair_device", { peerId });
}

export async function getNearbyDevices(): Promise<PeerInfo[]> {
  return invoke("get_nearby_devices");
}
