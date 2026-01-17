# Presenter Plugin - Laser Pointer Implementation Guide

## Current Status

✅ **Completed:**
- Laser pointer overlay module interface (`laser_pointer.rs`)
- Integration with Presenter plugin
- Position tracking and movement accumulation
- Show/hide lifecycle management
- Comprehensive test coverage

⏳ **Pending: Visual Overlay Rendering**

The laser pointer module currently **logs** pointer movements but does not display a visual overlay on screen. Full implementation requires one of the following approaches.

## Implementation Options

### Option 1: Wayland Layer Shell (Recommended for COSMIC)

**Dependencies:**
```toml
[dependencies]
smithay-client-toolkit = "0.18"
wayland-client = "0.31"
wayland-protocols-wlr = "0.3"  # For zwlr_layer_shell_v1
```

**Implementation Steps:**

1. **Create Wayland connection in `LaserPointer::show()`:**
   ```rust
   use smithay_client_toolkit::{
       compositor::{CompositorHandler, CompositorState},
       delegate_compositor, delegate_layer, delegate_registry,
       output::{OutputHandler, OutputState},
       registry::{ProvidesRegistryState, RegistryState},
       shell::wlr_layer::{LayerShell, LayerSurface, LayerSurfaceBuilder},
   };

   let conn = wayland_client::Connection::connect_to_env()?;
   let (globals, event_queue) = registry_queue_init(&conn)?;
   ```

2. **Create layer-shell surface:**
   ```rust
   let layer_surface = LayerSurfaceBuilder::new()
       .layer(Layer::Overlay)  // Topmost layer
       .namespace("cosmic-connect-laser-pointer")
       .size((40, 40))  // Dot size
       .exclusive_zone(-1)  // Don't reserve space
       .keyboard_interactivity(KeyboardInteractivity::None)
       .build();
   ```

3. **Render laser pointer dot:**
   ```rust
   use tiny_skia::{Paint, Pixmap};

   let mut pixmap = Pixmap::new(40, 40).unwrap();
   let mut paint = Paint::default();
   paint.set_color_rgba8(255, 0, 0, 204);  // Semi-transparent red

   pixmap.fill_circle(20.0, 20.0, 20.0, &paint);
   ```

4. **Update position in `move_by()`:**
   ```rust
   layer_surface.set_position(
       self.position.0 as i32,
       self.position.1 as i32
   );
   layer_surface.commit();
   ```

**Estimated Effort:** 4-6 hours
**Binary Size Impact:** +500KB (wayland dependencies)

### Option 2: Separate Overlay Service (DBus-based)

**Architecture:**
```
Presenter Plugin  →  DBus Signal  →  Laser Pointer Service
                                       ↓
                                  Wayland Overlay
```

**New Binary:** `cosmic-connect-laser-pointer`

**DBus Interface:**
```xml
<interface name="org.cosmic.Connect.LaserPointer">
  <method name="Show"/>
  <method name="Hide"/>
  <method name="Move">
    <arg name="dx" type="d" direction="in"/>
    <arg name="dy" type="d" direction="in"/>
  </method>
  <method name="SetPosition">
    <arg name="x" type="d" direction="in"/>
    <arg name="y" type="d" direction="in"/>
  </method>
</interface>
```

**Benefits:**
- Separation of concerns
- Can be used by other applications
- Easier to manage Wayland lifecycle
- Optional dependency (doesn't require daemon rebuild)

**Drawbacks:**
- Additional binary to maintain
- DBus overhead
- Service coordination complexity

**Estimated Effort:** 6-8 hours

### Option 3: COSMIC Compositor Integration (Future)

Wait for official COSMIC compositor APIs for overlays. This may provide:
- Native overlay support
- Proper theme integration
- Permission management through COSMIC Settings
- Better integration with COSMIC's security model

**Estimated Effort:** Unknown (depends on API availability)

## Quick Start for Implementation

### 1. Choose Approach

For immediate results, use **Option 1 (Wayland Layer Shell)**.

### 2. Add Dependencies

Update `cosmic-connect-protocol/Cargo.toml`:
```toml
[dependencies]
# Existing dependencies...

# Wayland overlay (optional, behind feature flag)
smithay-client-toolkit = { version = "0.18", optional = true }
wayland-client = { version = "0.31", optional = true }
wayland-protocols-wlr = { version = "0.3", optional = true }
tiny-skia = { version = "0.11", optional = true }

[features]
laser-pointer-overlay = [
    "smithay-client-toolkit",
    "wayland-client",
    "wayland-protocols-wlr",
    "tiny-skia"
]
```

### 3. Implement WaylandLaserPointer

Replace stub methods in `laser_pointer.rs`:

```rust
#[cfg(feature = "laser-pointer-overlay")]
pub struct LaserPointer {
    config: LaserPointerConfig,
    active: bool,
    position: (f64, f64),
    wayland_ctx: Option<WaylandContext>,
}

#[cfg(feature = "laser-pointer-overlay")]
struct WaylandContext {
    conn: wayland_client::Connection,
    layer_surface: LayerSurface,
    pixmap: tiny_skia::Pixmap,
}

#[cfg(feature = "laser-pointer-overlay")]
impl LaserPointer {
    pub fn show(&mut self) {
        if self.wayland_ctx.is_none() {
            self.wayland_ctx = Some(Self::create_wayland_overlay()?);
        }
        self.active = true;
    }

    fn create_wayland_overlay() -> Result<WaylandContext> {
        // Implementation here
    }
}
```

### 4. Test

```bash
# Build with feature flag
cargo build --features laser-pointer-overlay

# Test presenter plugin
cargo test --package cosmic-connect-protocol --features laser-pointer-overlay presenter
```

## Security Considerations

- Layer shell overlay requires Wayland compositor support
- Verify overlay permissions on security-focused compositors
- Add user consent dialog before first use
- Respect accessibility settings (high contrast mode, reduced motion)

## Performance Notes

- Overlay window should use minimal resources
- Consider fade-out timer to hide after inactivity (currently set to 2000ms)
- Debounce rapid position updates if needed

## References

- [Smithay Client Toolkit Documentation](https://smithay.github.io/client-toolkit/)
- [wlr-layer-shell Protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1)
- [KDE Connect Presenter Plugin](https://github.com/KDE/kdeconnect-kde/tree/master/plugins/presenter)
- [COSMIC Desktop](https://github.com/pop-os/cosmic-epoch)

## Current Behavior

Without visual overlay implementation:
- ✅ Presenter plugin receives pointer events
- ✅ Position tracking works correctly
- ✅ Show/hide lifecycle managed
- ❌ No visual indication on screen (logs only)

To see laser pointer events in action:
```bash
# Run daemon with debug logging
RUST_LOG=debug cosmic-connect-daemon

# Look for log messages like:
# [DEBUG] Presenter pointer moved: dx=10.5, dy=-5.2
# [INFO] Laser pointer overlay shown at (10.5, -5.2)
```
