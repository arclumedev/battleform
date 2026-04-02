## 1. Dependencies & Feature Flags

- [ ] 1.1 Add `bevy_remote` as a workspace dependency in `game/Cargo.toml` (version 0.18, default-features = false)
- [ ] 1.2 Add `bevy_brp_extras` as a dependency in `game/crates/bf_game/Cargo.toml`
- [ ] 1.3 Add `brp` feature flag to `game/crates/bf_game/Cargo.toml` that enables `bevy/bevy_remote`, `bevy_brp_extras`
- [ ] 1.4 Make the `hot` feature include `brp` (so hot reload builds always have inspection)
- [ ] 1.5 Verify WASM build still compiles without BRP: `cargo check -p bf_game --no-default-features --features wasm --target wasm32-unknown-unknown`

## 2. Plugin Integration

- [ ] 2.1 Add `RemotePlugin` and `BrpExtrasPlugin` to the native app builder behind `#[cfg(feature = "brp")]` in `game/crates/bf_game/src/main.rs`
- [ ] 2.2 Verify native build with BRP compiles: `cargo check -p bf_game --no-default-features --features native,brp`
- [ ] 2.3 Verify native build without BRP still compiles: `cargo check -p bf_game --no-default-features --features native`
- [ ] 2.4 Run the native client with BRP and confirm HTTP endpoint responds on port 15702

## 3. MCP Server Configuration

- [ ] 3.1 Install `bevy_brp_mcp` binary: `cargo install bevy_brp_mcp`
- [ ] 3.2 Add `bevy_brp_mcp` as an MCP server in `.claude/settings.json` for this project
- [ ] 3.3 Verify Claude Code can connect to the MCP server and list available BRP tools

## 4. Launch Config & Workflow

- [ ] 4.1 Update "Game: Native (Hot Reload)" launch config to use `hot` feature (already includes `brp`)
- [ ] 4.2 Add "Game: Native (Debug + BRP)" launch config with `--features native,brp`
- [ ] 4.3 Update CLAUDE.md with BRP usage instructions (how to start native with BRP, what tools are available)

## 5. Validation

- [ ] 5.1 Start native client with BRP, use Claude Code to `world_query` for terrain entities — verify count matches expected 32x32 grid
- [ ] 5.2 Use `brp_extras_screenshot` to capture a frame — verify PNG is returned
- [ ] 5.3 Use `brp_extras_send_keys` to simulate WASD — verify camera moves
- [ ] 5.4 Run `cargo clippy` on native+brp — no warnings
- [ ] 5.5 Run `cargo clippy` on wasm — no warnings (BRP not compiled)
