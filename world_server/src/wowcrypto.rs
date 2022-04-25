use crate::prelude::*;
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::rc4::Rc4;
use crypto::sha1::Sha1;
use crypto::symmetriccipher::SynchronousStreamCipher;

static S: [u8; 16] = [
    0xC2, 0xB3, 0x72, 0x3C, 0xC6, 0xAE, 0xD9, 0xB5, 0x34, 0x3C, 0x53, 0xEE, 0x2F, 0x43, 0x67, 0xCE,
];
static R: [u8; 16] = [
    0xCC, 0x98, 0xAE, 0x04, 0xE8, 0x97, 0xEA, 0xCA, 0x12, 0xDD, 0xC0, 0x93, 0x42, 0x91, 0x53, 0x57,
];

#[derive(Default)]
pub struct ClientCrypto {
    encrypter: Option<Rc4>,
    decrypter: Option<Rc4>,
}

impl ClientCrypto {
    pub fn new() -> Self {
        Self {
            encrypter: None,
            decrypter: None,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.encrypter.is_some() && self.decrypter.is_some()
    }

    pub fn initialize(&mut self, sess_key: &[u8]) -> Result<()> {
        assert_eq!(sess_key.len(), 40);

        let mut sha1 = Sha1::new();
        let mut hmac = Hmac::new(sha1, &S);

        //Decrypt key (c->s)
        hmac.input(sess_key);
        let hash_result = hmac.result();
        let decrypt_hash = hash_result.code();

        sha1.reset();
        //Encrypt key (s->c)
        hmac = Hmac::new(sha1, &R);
        hmac.input(sess_key);
        let hash_result = hmac.result();
        let encrypt_hash = hash_result.code();

        self.encrypter = Some(Rc4::new(encrypt_hash));
        self.decrypter = Some(Rc4::new(decrypt_hash));

        //Process some zero bytes to prevent some attack idk
        let mut void_output = [0u8; 1024];
        self.encrypter.as_mut().unwrap().process(&[0; 1024], &mut void_output);
        self.decrypter.as_mut().unwrap().process(&[0; 1024], &mut void_output);

        Ok(())
    }

    pub fn encrypt(&mut self, data: &mut Vec<u8>) -> Result<()> {
        assert_eq!(data.len(), 4);
        self.encrypter.as_mut().unwrap().process(&data.clone(), data);
        Ok(())
    }

    pub fn decrypt(&mut self, data: &mut Vec<u8>) -> Result<()> {
        assert_eq!(data.len(), 6);
        self.decrypter.as_mut().unwrap().process(&data.clone(), data);
        Ok(())
    }
}
