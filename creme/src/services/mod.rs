mod dev_service;
mod release_service;

#[cfg(debug_assertions)]
pub use dev_service::CremeDevService as CremeService;
#[cfg(not(debug_assertions))]
pub use release_service::CremeReleaseService as CremeService;
