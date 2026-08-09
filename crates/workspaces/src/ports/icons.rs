use std::sync::Arc;

/// The icon side of the bar: resolve an application's icon as encoded PNG
/// bytes. `None` degrades to the initial-letter tile — icons must never block
/// the pills.
pub trait AppIconProvider: Send + Sync {
    fn icon_png(&self, bundle_id: Option<&str>, app_name: &str) -> Option<Arc<Vec<u8>>>;
}
