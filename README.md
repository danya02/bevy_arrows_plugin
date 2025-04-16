# bevy_arrows_plugin

A Bevy plugin for drawing arrows next to components.

Very WIP, do not rely on the API remaining stable.

To use, simply add the plugin to the app:

```rust
App::new().add_plugins(BevyArrowsPlugin)
```

Then, when spawning an entity, add the [`VecArrow`] component:

```rust
commands.spawn((
    todo!(),
    VecArrow::new(Vec3::new(2.0, 2.0, 0.0), TargetCoordinateSpace::Local)
        .with_color(Color::linear_rgb(1.0, 1.0, 0.0)),
));
```