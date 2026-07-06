# Dual-Target Strategy: 2D Desktop vs. VR

The Communication Class engine employs a strict dual-target architecture leveraging Bevy's feature flags to separate **Rapid Development / Free Marketing** from the **Premium Monetized Product**.

## 🖥️ 2D Mode (Desktop / Web)
**Purpose**: Rapid testing of core psycholinguistic mechanics, UI iteration, and free marketing demos.
**Compilation**: `cargo run --features desktop` or compiled to WebAssembly.

- **Game Mechanics Focused**: This mode is stripped of complex physics interactions, allowing the developers and players to quickly click through tutorials, collect words, and trigger battles.
- **Screen-Space UI**: Uses standard `bevy_ui` (`Node`, `Text`, absolute positioning) which renders directly to the user's screen.
- **Marketing Value**: Easily embeddable in web browsers as a pitch deck or free tier. It acts as the "learning engine" where we master the communication mechanics *before* paying the performance penalty of rendering a physical world.

## 🥽 VR Mode (OpenXR)
**Purpose**: The premium, monetized EdTech experience.
**Compilation**: `cargo run --features xr` (typically cross-compiled for Android/Quest).

- **Physical World Focused**: Replaces mouse clicks with `PinchEvents` and distance-based raycasting.
- **Spatial UI**: Removes all screen-space `bevy_ui` components. Menus, tutorials, and interactions are rendered as Holographic 3D panels that exist in the physical space of the classroom.
- **Monetization**: This is the flagship product sold to school districts and parents, wrapping the proven 2D mechanics into an immersive physical space.
