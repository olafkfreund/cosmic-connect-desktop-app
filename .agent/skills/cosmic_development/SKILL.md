---
name: Cosmic Desktop Development
description: Best practices and guidelines for developing applications and applets for the COSMIC Desktop Environment.
---

# Cosmic Desktop Development

This skill provides guidelines and best practices for developing applications and applets for the System76 COSMIC Desktop Environment (COSMIC DE).

## 1. Overview & Technology Stack

COSMIC applications are built using **Rust** and **libcosmic**.

- **Language**: Rust (Safe, fast, concurrent)
- **GUI Toolkit**: `libcosmic` (Built on top of `iced`, a cross-platform GUI library inspired by Elm)
- **Design System**: COSMIC Design System (Theming, consistent widgets)

## 2. Project Setup

Always start strict adherence to COSMIC standards by using official templates.

### App Template

For standalone applications:

```bash
cargo generate --git https://github.com/pop-os/cosmic-app-template
```

### Applet Template

For panel applets:

```bash
cargo generate --git https://github.com/pop-os/cosmic-applet-template
```

## 3. Core Development Principles

### GUI Architecture (The Elm Architecture)

`libcosmic` (and `iced`) follows the Model-View-Update (MVU) pattern:

1.  **State (Model)**: The data structure describing the application's state.
2.  **Message**: Enum variants representing user actions or events.
3.  **Update**: A pure function (`fn update(&mut state, message)`) that modifies the state based on a message.
4.  **View**: A pure function (`fn view(&state) -> Element`) that renders the UI based on the state.

### Best Practices

- **Use `libcosmic` Widgets**: Always prefer `libcosmic` widgets over raw `iced` widgets when available to ensure consistent styling and integration with the desktop theme.
- **Modular Design**: Separate your `update`, `view`, and `state` logic. For complex apps, break components into sub-modules with their own MVU cycles.
- **Configuration**: Integrate with `cosmic-config` for handling user settings. This ensures your app's settings persist and respect system-wide overrides.
- **Theming**: Do not hardcode colors! Use the semantic colors provided by the `cosmic-theme` (e.g., `theme.palette.primary`, `theme.palette.background`). This ensures your app looks correct in both Light and Dark modes.

## 4. Applet Specifics

- **Composability**: Applets often live in the panel. Keep them lightweight.
- **Popup vs Embedded**: Decide if your applet needs a popover menu (like WiFi) or just an icon/text (like a clock).
- **Responsiveness**: Applets must handle panel resizing gracefully.
- **Wayland Integration**: COSMIC is Wayland-first. Ensure your applet interacts correctly with the compositor layers if doing custom windowing.

## 5. Important Libraries (The "Cosmic Stack")

- `cosmic-text`: Advanced text shaping and rendering.
- `cosmic-config`: Type-safe configuration management.
- `cosmic-theme`: Access to system colors and metrics.
- `cosmic-comp`: The compositor (useful for reference if interacting with window management).

## 6. Resources

- **libcosmic Source**: https://github.com/pop-os/libcosmic
- **Official Examples**: Check `examples/` in the libcosmic repo.
- **Iced Documentation**: https://docs.rs/iced (Foundational knowledge)
- **System76 Dev Docs**: https://github.com/pop-os/cosmic-epoch

## 7. Troubleshooting

- **"Component not found"**: Ensure you have `libcosmic` features enabled in `Cargo.toml`.
- **Theming issues**: Verify you are taking `Theme` as an argument in your `view` function and passing it correctly.
