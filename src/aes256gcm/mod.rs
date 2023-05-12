/*!
aes-256-cbc module

This library provides user-friendly utilities for performing AES-256-CBC operations.

Currenly supports:

- key derivation with password
- encryption
- decryption

# Example

This example shows how to create a "standard" printer and execute a search.

```
use toolz::aes256cbc::{Key, Config};

let config = Config::from_vec(&[100, 200, 300]);

let password = String::from("I <3 Nickelback");
let key = Key::from_password(&password.as_bytes(), &config);

let plaintext = b"Some secret information";
let cyphertext = key.encrypt(plaintext).ok().expect("encryption failed");

let decrypted = key.decrypt(&cyphertext).ok().expect("decryption failed");

assert_eq!((*plaintext).to_vec(), decrypted);
```
 */



#[cfg(test)]
mod tests {
extern crate aes_gcm;
extern crate crypto;


use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce // Or `Aes128Gcm`
};

    #[test]
    fn test_encrypt_and_decrypt() {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(b"unique nonce"); // 96-bits; unique per message
        let ciphertext: Vec<u8> = match cipher.encrypt(nonce, b"plaintext message".as_ref()) {
            Ok(p) => p,
            Err(e) => {
                panic!("{}", e)
            },
        };
        let plaintext = match cipher.decrypt(nonce, ciphertext.as_ref()) {
            Ok(p) => p,
            Err(e) => {
                panic!("{}", e)
            },
        };
        assert_eq!(&plaintext, b"plaintext message");
    }
}
