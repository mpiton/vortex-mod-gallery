//! Guards the `[config]` contract: keys the plugin reads via `get_config`
//! must be declared in `plugin.toml` so the host can validate and expose them.

const MANIFEST: &str = include_str!("../plugin.toml");

#[test]
fn manifest_declares_every_config_key_read_by_the_plugin() {
    let (_, config_section) = MANIFEST
        .split_once("[config]")
        .expect("plugin.toml must have a [config] section");
    for key in [
        "min_resolution",
        "auto_name",
        "imgur_client_id",
        "flickr_api_key",
    ] {
        let declared = config_section.lines().any(|line| {
            line.split_once('=')
                .is_some_and(|(name, _)| name.trim() == key)
        });
        assert!(
            declared,
            "plugin.toml [config] is missing '{key}', which plugin_api.rs reads via get_config"
        );
    }
}
