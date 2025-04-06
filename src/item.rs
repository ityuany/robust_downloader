use typed_builder::TypedBuilder;

#[derive(Debug, Clone)]
pub enum Integrity {
  #[cfg(feature = "md5")]
  MD5(String),
  #[cfg(feature = "sha1")]
  SHA1(String),
  #[cfg(feature = "sha2")]
  SHA256(String),
  #[cfg(feature = "sha2")]
  SHA512(String),
  #[cfg(feature = "sha3")]
  SHA3_256(String),
  #[cfg(feature = "blake2")]
  Blake2b(String),
  #[cfg(feature = "blake2")]
  Blake2s(String),
  #[cfg(feature = "blake3")]
  Blake3(String),
}

impl Integrity {
  pub fn value(&self) -> &str {
    match self {
      #[cfg(feature = "md5")]
      Integrity::MD5(value) => value,
      #[cfg(feature = "sha1")]
      Integrity::SHA1(value) => value,
      #[cfg(feature = "sha2")]
      Integrity::SHA256(value) => value,
      #[cfg(feature = "sha2")]
      Integrity::SHA512(value) => value,
      #[cfg(feature = "sha3")]
      Integrity::SHA3_256(value) => value,
      #[cfg(feature = "blake2")]
      Integrity::Blake2b(value) => value,
      #[cfg(feature = "blake2")]
      Integrity::Blake2s(value) => value,
      #[cfg(feature = "blake3")]
      Integrity::Blake3(value) => value,
    }
  }

  pub fn algorithm(&self) -> hashery::Algorithm {
    match self {
      #[cfg(feature = "md5")]
      Integrity::MD5(_) => hashery::Algorithm::MD5,
      #[cfg(feature = "sha1")]
      Integrity::SHA1(_) => hashery::Algorithm::SHA1,
      #[cfg(feature = "sha2")]
      Integrity::SHA256(_) => hashery::Algorithm::SHA256,
      #[cfg(feature = "sha2")]
      Integrity::SHA512(_) => hashery::Algorithm::SHA512,
      #[cfg(feature = "sha3")]
      Integrity::SHA3_256(_) => hashery::Algorithm::SHA3_256,
      #[cfg(feature = "blake2")]
      Integrity::Blake2b(_) => hashery::Algorithm::Blake2b,
      #[cfg(feature = "blake2")]
      Integrity::Blake2s(_) => hashery::Algorithm::Blake2s,
      #[cfg(feature = "blake3")]
      Integrity::Blake3(_) => hashery::Algorithm::Blake3,
    }
  }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct DownloadItem<U, P> {
  pub url: U,
  pub target: P,

  #[builder(default = None, setter(strip_option))]
  pub integrity: Option<Integrity>,
}
