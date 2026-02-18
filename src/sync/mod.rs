pub mod download;
pub mod products;
pub mod upload;

#[macro_export]
macro_rules! get_toml_value {
    ($doc:expr, $name:expr) => {
        $doc[$name].as_table_mut().cloned().unwrap_or_default()
    };
}
