// Input handler module for domain-based keyboard navigation
// Handles WASD navigation, domain switching, and spatial cursor movement

pub mod types;
pub mod domain_navigator;
pub mod spatial;

pub use domain_navigator::DomainNavigator;
pub use types::*;



