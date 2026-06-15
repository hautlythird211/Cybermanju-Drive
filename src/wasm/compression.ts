import { withWasm } from './bridge'

export async function compressLz4(data: Uint8Array): Promise<Uint8Array> {
  return withWasm(m => m.compress_lz4(data))
}

export async function decompressLz4(data: Uint8Array): Promise<Uint8Array> {
  return withWasm(m => m.decompress_lz4(data))
}

export async function compressBrotli(
  data: Uint8Array,
  quality: number = 6
): Promise<Uint8Array> {
  return withWasm(m => m.compress_brotli(data, quality))
}

export async function decompressBrotli(
  data: Uint8Array
): Promise<Uint8Array> {
  return withWasm(m => m.decompress_brotli(data))
}

export async function probeLz4Ratio(
  data: Uint8Array
): Promise<number> {
  return withWasm(m => m.compress_lz4_probe_ratio(data))
}
