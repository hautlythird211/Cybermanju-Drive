use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use ml_dsa::{Generate, KeyExport, Keypair, MlDsa65, Signer, SigningKey, VerifyingKey};
use sha2::Sha512;
use wasm_bindgen::prelude::*;
use x25519_dalek::{PublicKey, StaticSecret};

use ml_kem::{DecapsulationKey1024, EncapsulationKey1024, MlKem1024, KeySizeUser};
use ml_kem::kem::{Decapsulate, Encapsulate, Kem};
use ml_kem::kem::Ciphertext;

type HmacSha512 = Hmac<Sha512>;

#[wasm_bindgen]
pub fn blake3_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

#[wasm_bindgen]
pub fn chacha20_generate_key() -> Vec<u8> {
    use chacha20poly1305::aead::OsRng as AeadOsRng;
    use rand_core::RngCore;
    let mut key = [0u8; 32];
    AeadOsRng.fill_bytes(&mut key);
    key.to_vec()
}

#[wasm_bindgen]
pub fn chacha20_generate_nonce() -> Vec<u8> {
    use rand_core::RngCore;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce.to_vec()
}

#[wasm_bindgen]
pub fn chacha20_encrypt(key: &[u8], nonce: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, JsValue> {
    let key_arr: [u8; 32] = key
        .try_into()
        .map_err(|_| JsValue::from_str("Key must be 32 bytes"))?;
    let nonce_arr: [u8; 12] = nonce
        .try_into()
        .map_err(|_| JsValue::from_str("Nonce must be 12 bytes"))?;

    let cipher = ChaCha20Poly1305::new_from_slice(&key_arr)
        .map_err(|e| JsValue::from_str(&format!("Failed to create cipher: {}", e)))?;
    let nonce = Nonce::from(nonce_arr);

    cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| JsValue::from_str(&format!("Encryption failed: {}", e)))
}

#[wasm_bindgen]
pub fn chacha20_decrypt(key: &[u8], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, JsValue> {
    let key_arr: [u8; 32] = key
        .try_into()
        .map_err(|_| JsValue::from_str("Key must be 32 bytes"))?;
    let nonce_arr: [u8; 12] = nonce
        .try_into()
        .map_err(|_| JsValue::from_str("Nonce must be 12 bytes"))?;

    let cipher = ChaCha20Poly1305::new_from_slice(&key_arr)
        .map_err(|e| JsValue::from_str(&format!("Failed to create cipher: {}", e)))?;
    let nonce = Nonce::from(nonce_arr);

    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| JsValue::from_str(&format!("Decryption failed: {}", e)))
}

#[wasm_bindgen]
pub fn hkdf_derive(secret: &[u8], salt: &[u8], info: &[u8], length: usize) -> Vec<u8> {
    let hk = Hkdf::<sha2::Sha256>::new(Some(salt), secret);
    let mut okm = vec![0u8; length];
    hk.expand(info, &mut okm).expect("HKDF expand failed");
    okm
}

#[wasm_bindgen]
pub fn hmac_sha512(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = <HmacSha512 as Mac>::new_from_slice(key).expect("HMAC key should be valid");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

#[wasm_bindgen]
pub fn x25519_generate_keypair() -> Result<js_sys::Object, JsValue> {
    use rand_core::OsRng;

    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("privateKey"),
        &js_sys::Uint8Array::from(secret.to_bytes().as_slice()),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("publicKey"),
        &js_sys::Uint8Array::from(public.as_bytes().as_slice()),
    )?;
    Ok(obj.into())
}

#[wasm_bindgen]
pub fn x25519_shared_secret(private_key: &[u8], peer_public: &[u8]) -> Result<Vec<u8>, JsValue> {
    let private_arr: [u8; 32] = private_key
        .try_into()
        .map_err(|_| JsValue::from_str("Private key must be 32 bytes"))?;
    let public_arr: [u8; 32] = peer_public
        .try_into()
        .map_err(|_| JsValue::from_str("Public key must be 32 bytes"))?;

    let secret = StaticSecret::from(private_arr);
    let public = PublicKey::from(public_arr);
    let shared = secret.diffie_hellman(&public);
    Ok(shared.as_bytes().to_vec())
}

#[wasm_bindgen]
pub fn ml_dsa65_generate_keypair() -> Result<js_sys::Object, JsValue> {
    let signing_key = SigningKey::<MlDsa65>::generate();
    let verifying_key = signing_key.verifying_key();

    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("privateKey"),
        &js_sys::Uint8Array::from(signing_key.to_bytes().as_slice()),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("publicKey"),
        &js_sys::Uint8Array::from(verifying_key.to_bytes().as_slice()),
    )?;
    Ok(obj.into())
}

#[wasm_bindgen]
pub fn ml_dsa65_sign(message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, JsValue> {
    let seed = ml_dsa::Seed::try_from(private_key)
        .map_err(|_| JsValue::from_str("Invalid private key (must be 32-byte seed)"))?;
    let signing_key = SigningKey::<MlDsa65>::from_seed(&seed);
    let signature = signing_key.sign(message);
    let encoded = signature.encode();
    Ok(encoded.to_vec())
}

#[wasm_bindgen]
pub fn ml_dsa65_verify(
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> Result<bool, JsValue> {
    use signature::Verifier;

    let encoded_vk = ml_dsa::EncodedVerifyingKey::<MlDsa65>::try_from(public_key)
        .map_err(|_| JsValue::from_str("Invalid public key"))?;
    let verifying_key = VerifyingKey::<MlDsa65>::decode(&encoded_vk);

    let encoded_sig = ml_dsa::EncodedSignature::<MlDsa65>::try_from(signature)
        .map_err(|_| JsValue::from_str("Invalid signature"))?;
    let sig = ml_dsa::Signature::<MlDsa65>::decode(&encoded_sig)
        .ok_or_else(|| JsValue::from_str("Failed to decode signature"))?;

    Ok(verifying_key.verify(message, &sig).is_ok())
}

#[wasm_bindgen]
pub fn ml_kem1024_generate_keypair() -> Result<js_sys::Object, JsValue> {
    let (dk, ek) = MlKem1024::generate_keypair();
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("publicKey"),
        &js_sys::Uint8Array::from(ek.to_bytes().as_slice()),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("secretKey"),
        &js_sys::Uint8Array::from(dk.to_bytes().as_slice()),
    )?;
    Ok(obj.into())
}

#[wasm_bindgen]
pub fn ml_kem1024_encapsulate(public_key: &[u8]) -> Result<js_sys::Object, JsValue> {
    use core::convert::TryInto;
    let pk_arr: ml_kem::array::Array<u8, <EncapsulationKey1024 as ml_kem::KeySizeUser>::KeySize> = public_key
        .try_into()
        .map_err(|_| JsValue::from_str("Invalid ML-KEM-1024 public key: wrong length"))?;
    let ek = EncapsulationKey1024::new(&pk_arr)
        .map_err(|e| JsValue::from_str(&format!("Invalid ML-KEM-1024 public key: {:?}", e)))?;
    let (ciphertext, shared_secret) = ek.encapsulate();
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("ciphertext"),
        &js_sys::Uint8Array::from(ciphertext.as_slice()),
    )?;
    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("sharedSecret"),
        &js_sys::Uint8Array::from(shared_secret.as_slice()),
    )?;
    Ok(obj.into())
}

#[wasm_bindgen]
pub fn ml_kem1024_decapsulate(secret_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, JsValue> {
    use core::convert::TryInto;
    let seed: ml_kem::Seed = secret_key
        .try_into()
        .map_err(|_| JsValue::from_str("Invalid ML-KEM-1024 secret key"))?;
    let dk = DecapsulationKey1024::from_seed(seed);
    let ct: Ciphertext<MlKem1024> = ciphertext
        .try_into()
        .map_err(|_| JsValue::from_str("Invalid ML-KEM-1024 ciphertext"))?;
    let shared_secret = dk.decapsulate(&ct);
    Ok(shared_secret.to_vec())
}
