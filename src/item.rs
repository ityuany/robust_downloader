use hashery::Algorithm;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, TypedBuilder)]
pub struct IntegrityHash {
  pub expect: String,
  pub algorithm: Algorithm,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct DownloadItem<U, P> {
  pub url: U,
  pub target: P,

  #[builder(default = None, setter(strip_option))]
  pub integrity_hash: Option<IntegrityHash>,
}
