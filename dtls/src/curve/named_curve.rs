use rand_core::OsRng; // requires 'getrandom' feature

use util::Error;

use crate::errors::*;

// https://www.iana.org/assignments/tls-parameters/tls-parameters.xml#tls-parameters-8
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NamedCurve {
    P256 = 0x0017,
    P384 = 0x0018,
    X25519 = 0x001d,
    Unsupported,
}

impl From<u16> for NamedCurve {
    fn from(val: u16) -> Self {
        match val {
            0x0017 => NamedCurve::P256,
            0x0018 => NamedCurve::P384,
            0x001d => NamedCurve::X25519,
            _ => NamedCurve::Unsupported,
        }
    }
}

pub(crate) enum NamedCurvePrivateKey {
    EphemeralSecretP256(p256::ecdh::EphemeralSecret),
    EphemeralSecretP384(ring::agreement::EphemeralPrivateKey),
    StaticSecretX25519(x25519_dalek::StaticSecret),
}

pub struct NamedCurveKeypair {
    pub(crate) curve: NamedCurve,
    pub(crate) public_key: Vec<u8>,
    pub(crate) private_key: NamedCurvePrivateKey,
}

fn elliptic_curve_keypair(curve: NamedCurve) -> Result<NamedCurveKeypair, Error> {
    let (public_key, private_key) = match curve {
        NamedCurve::P256 => {
            let secret_key = p256::ecdh::EphemeralSecret::random(&mut OsRng);
            let public_key = p256::EncodedPoint::from(&secret_key);
            (
                public_key.as_bytes().to_vec(),
                NamedCurvePrivateKey::EphemeralSecretP256(secret_key),
            )
        }
        NamedCurve::P384 => {
            let rng = ring::rand::SystemRandom::new();
            let secret_key = ring::agreement::EphemeralPrivateKey::generate(&ring::agreement::ECDH_P256, &rng)?;
            let public_key = secret_key.compute_public_key()?;
            (
                public_key.as_ref().to_vec(),
                NamedCurvePrivateKey::EphemeralSecretP384(secret_key),
            )
        }
        NamedCurve::X25519 => {
            let secret_key = x25519_dalek::StaticSecret::new(rand_core::OsRng);
            let public_key = x25519_dalek::PublicKey::from(&secret_key);
            (
                public_key.as_bytes().to_vec(),
                NamedCurvePrivateKey::StaticSecretX25519(secret_key),
            )
        }
        _ => return Err(ERR_INVALID_NAMED_CURVE.clone()),
    };

    Ok(NamedCurveKeypair {
        curve,
        public_key,
        private_key,
    })
}

impl NamedCurve {
    pub fn generate_keypair(&self) -> Result<NamedCurveKeypair, Error> {
        match *self {
            NamedCurve::X25519 => elliptic_curve_keypair(NamedCurve::X25519),
            NamedCurve::P256 => elliptic_curve_keypair(NamedCurve::P256),
            NamedCurve::P384 => elliptic_curve_keypair(NamedCurve::P384),
            _ => Err(ERR_INVALID_NAMED_CURVE.clone()),
        }
    }
}
