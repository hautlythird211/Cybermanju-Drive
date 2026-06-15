import { withWasm } from './bridge'

export async function blake3Hash(data: Uint8Array): Promise<string> {
  return withWasm(m => m.blake3_hash(data))
}

export async function generateEncryptionKey(): Promise<Uint8Array> {
  return withWasm(m => m.chacha20_generate_key())
}

export async function generateNonce(): Promise<Uint8Array> {
  return withWasm(m => m.chacha20_generate_nonce())
}

export async function encryptData(
  key: Uint8Array,
  nonce: Uint8Array,
  plaintext: Uint8Array
): Promise<Uint8Array> {
  return withWasm(m => m.chacha20_encrypt(key, nonce, plaintext))
}

export async function decryptData(
  key: Uint8Array,
  nonce: Uint8Array,
  ciphertext: Uint8Array
): Promise<Uint8Array> {
  return withWasm(m => m.chacha20_decrypt(key, nonce, ciphertext))
}

export async function hkdfDerive(
  secret: Uint8Array,
  salt: Uint8Array,
  info: Uint8Array,
  length: number
): Promise<Uint8Array> {
  return withWasm(m => m.hkdf_derive(secret, salt, info, length))
}

export async function hmacSha512(
  key: Uint8Array,
  data: Uint8Array
): Promise<Uint8Array> {
  return withWasm(m => m.hmac_sha512(key, data))
}

export async function generateX25519Keypair(): Promise<{
  privateKey: Uint8Array
  publicKey: Uint8Array
}> {
  return withWasm(m => {
    const kp = m.x25519_generate_keypair()
    return {
      privateKey: new Uint8Array(kp.privateKey as ArrayBuffer),
      publicKey: new Uint8Array(kp.publicKey as ArrayBuffer),
    }
  })
}

export async function x25519SharedSecret(
  privateKey: Uint8Array,
  peerPublic: Uint8Array
): Promise<Uint8Array> {
  return withWasm(m => m.x25519_shared_secret(privateKey, peerPublic))
}

export async function generateMlDsa65Keypair(): Promise<{
  privateKey: Uint8Array
  publicKey: Uint8Array
}> {
  return withWasm(m => {
    const kp = m.ml_dsa65_generate_keypair()
    return {
      privateKey: new Uint8Array(kp.privateKey as ArrayBuffer),
      publicKey: new Uint8Array(kp.publicKey as ArrayBuffer),
    }
  })
}

export async function mlDsa65Sign(
  message: Uint8Array,
  privateKey: Uint8Array
): Promise<Uint8Array> {
  return withWasm(m => m.ml_dsa65_sign(message, privateKey))
}

export async function mlDsa65Verify(
  message: Uint8Array,
  signature: Uint8Array,
  publicKey: Uint8Array
): Promise<boolean> {
  return withWasm(m => m.ml_dsa65_verify(message, signature, publicKey))
}

export async function generateUuid(): Promise<string> {
  return withWasm(m => m.generate_uuid())
}

export async function hashFileMeta(
  name: string,
  size: number,
  modified: string
): Promise<string> {
  return withWasm(m => m.hash_file_meta(name, size, modified))
}
