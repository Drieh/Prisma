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
prismae = "0.3.1"
```

## Example

This example creates a window with a single rounded orange node.

```rust
fn main() -> Result<(), PrismaError> {
    let mut scene = Scene::new();

    scene
        .new_node()
        .bg_color(Color::ORANGE)
        .size(150, 150)
        .border_radius(25)
        .position(100, 100)
        .content("Hello World");

    scene.on_event::<WindowCloseRequest>(|context, _| {
        context.close(0);
    });

    let mut prisma = Prisma::init()?;
    prisma.add_window(
        WindowBuilder::new("Example")
            .position_centered()
            .resizable()
            .size(800, 400),
        scene,
    )?;
    prisma.run()?;
    Ok(())
}
```

## License

MIT