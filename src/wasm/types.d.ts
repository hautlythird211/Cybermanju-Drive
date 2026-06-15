declare module 'cybermanju-drive-wasm' {
  export function init(): void
  export function generate_uuid(): string
  export function now_utc(): string
  export function hash_file_meta(name: string, size: number, modified: string): string

  // Crypto
  export function blake3_hash(data: Uint8Array): string
  export function chacha20_generate_key(): Uint8Array
  export function chacha20_generate_nonce(): Uint8Array
  export function chacha20_encrypt(key: Uint8Array, nonce: Uint8Array, plaintext: Uint8Array): Uint8Array
  export function chacha20_decrypt(key: Uint8Array, nonce: Uint8Array, ciphertext: Uint8Array): Uint8Array
  export function hkdf_derive(secret: Uint8Array, salt: Uint8Array, info: Uint8Array, length: number): Uint8Array
  export function hmac_sha512(key: Uint8Array, data: Uint8Array): Uint8Array
  export function x25519_generate_keypair(): { privateKey: ArrayBuffer; publicKey: ArrayBuffer }
  export function x25519_shared_secret(privateKey: Uint8Array, peerPublic: Uint8Array): Uint8Array
  export function ml_dsa65_generate_keypair(): { privateKey: ArrayBuffer; publicKey: ArrayBuffer }
  export function ml_dsa65_sign(message: Uint8Array, privateKey: Uint8Array): Uint8Array
  export function ml_dsa65_verify(message: Uint8Array, signature: Uint8Array, publicKey: Uint8Array): boolean

  // Compression
  export function compress_lz4(data: Uint8Array): Uint8Array
  export function decompress_lz4(data: Uint8Array): Uint8Array
  export function compress_brotli(data: Uint8Array, quality: number): Uint8Array
  export function decompress_brotli(data: Uint8Array): Uint8Array
  export function compress_lz4_probe_ratio(data: Uint8Array): number

  // SyncEngine
  export class SyncEngine {
    constructor()
    add_file(path: string, size_bytes: number, backend: string): any
    mark_synced(file_id: string): void
    mark_error(file_id: string, error: string): void
    mark_changed(file_id: string): void
    remove_file(file_id: string): void
    get_entries(): any
    get_state(): any
    get_pending_count(): number
    get_error_count(): number
    get_synced_count(): number
    has_changes(): boolean
    reset(): void
    to_json(): string
    from_json(json: string): SyncEngine
  }

  // VirtualDrive
  export class VirtualDrive {
    constructor()
    create_file(name: string, file_type: string, parent_id: string | null): { id: string }
    delete_file(file_id: string): void
    get_file(file_id: string): any
    list_files(parent_id: string | null): any[]
    search_files(query: string): any[]
    rename_file(file_id: string, new_name: string): void
    move_file(file_id: string, new_parent_id: string | null): void
    set_file_size(file_id: string, size_bytes: number): void
    set_file_tags(file_id: string, tags: string[]): void
    toggle_star(file_id: string): boolean
    get_starred_files(): any[]
    get_geo_files(): any[]
    get_quota(): { usedBytes: number; totalBytes: number; fileCount: number; folderCount: number }
    get_all_files(): any[]
    file_count(): number
    folder_count(): number
    total_bytes(): number
    to_json(): string
    from_json(json: string): VirtualDrive
  }
}
