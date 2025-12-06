pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

use std::sync::Arc;
use application::use_cases::ExportUseCase;
use infrastructure::auth::JwtHandler;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub jwt_handler: Arc<JwtHandler>,
    pub use_case: Arc<ExportUseCase>,
}
