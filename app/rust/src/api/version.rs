#[flutter_rust_bridge::frb(sync)]
pub fn core_version() -> String {
    legato_engine::CORE_VERSION.to_string()
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
