# Changelog

All notable changes to vortex-mod-gallery will be documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [1.1.0] - 2026-07-17

### Added
- Declared `imgur_client_id` and `flickr_api_key` in the `[config]` manifest
  section so the host can validate and persist them; the plugin already reads
  both through `get_config` (MAT-134 R-01).
- Manifest test pinning every config key read by `plugin_api.rs` to a
  `[config]` declaration, and an ABI smoke assertion proving a configured
  Imgur client id travels from `get_config` into the `Client-ID`
  Authorization header.

## [1.0.0] - 2026-04-15

### Added
- Initial release: Imgur, Flickr, and generic HTML gallery extraction with
  `can_handle`, `supports_playlist`, `extract_links`, `extract_generic`, and
  `is_http_url` exports.
