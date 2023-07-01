pub use creme_macros::asset;
pub use creme_macros::service;

pub mod service;

#[macro_export]
macro_rules! is_release {
    () => {
        env!("CREME_RELEASE_MODE") == "release"
    };
}

#[macro_export]
macro_rules! is_development {
    () => {
        env!("CREME_RELEASE_MODE") == "development"
    };
}
