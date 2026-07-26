# Prisma

A simple UI framework for Rust built on SDL3.

## Features

- Scene system
- Node hierarchy
- Event-driven architecture
- Multiple windows
- Node state
- Lifecycle events

## Installation

```toml
[dependencies]
prismae = "0.2.0"
```

## Example

This example creates a window with a single rounded orange node.

```rust
fn main() -> Result<(), PrismaError> {
    let mut scene = Scene::new();

    scene
        .new_node()
        .bg_color(Color::rgb(255, 100, 50))
        .size(150, 150)
        .border_radius(25)
        .position(100, 100);

    scene.on_event(
        EventType::Window(WindowEventType::CloseRequest),
        |context| context.close(0),
    );
    Prisma::builder()?
        .window(
            WindowBuilder::new("Example")
                .position_centered()
                .resizable()
                .size(800, 400),
            scene,
        )?
        .build()?
        .run()?;
    Ok(())
}
```

## License

MIT