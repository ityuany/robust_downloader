use typed_builder::TypedBuilder;

#[cfg(any(
  feature = "md5",
  feature = "sha1",
  feature = "sha2",
  feature = "sha3",
  feature = "blake2",
  feature = "blake3"
))]
use crate::integrity::Integrity;

#[derive(Debug, Clone, TypedBuilder)]
pub struct DownloadItem<U, P> {
  pub url: U,
  pub target: P,

  #[cfg(any(
    feature = "md5",
    feature = "sha1",
    feature = "sha2",
    feature = "sha3",
    feature = "blake2",
    feature = "blake3"
  ))]
  #[builder(default = None, setter(strip_option))]
  pub integrity: Option<Integrity>,

  #[builder(default = None, setter(strip_option))]
  pub integrity_file: Option<()>,
}
