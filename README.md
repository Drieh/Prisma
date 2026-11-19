# Prisma

A simple UI framework for Rust built on SDL3.

## Features

- Scene system
- Node hierarchy
- Event-driven architecture
- Multiple windows
- Node state
- Lifecycle events
- Text with custom fonts

## Installation

```toml
[dependencies]
prismae = "0.3.2"
```

## Example

This example creates a window with a single lime node.

```rust
fn main() -> Result<(), PrismaError> {
    let mut prisma = Prisma::init()?;

    let roboto_font = prisma.register::<Font>("examples/assets/font/Roboto.ttf")?;

    let mut scene = Scene::new();

    scene
        .new_node()
        .style(move |style| {
            style
                .background(|bg| {
                    bg.color(Color::LIME);
                })
                .border(|br| {
                    br.radius(15);
                })
                .text(move |txt| {
                    txt.content("hola".into())
                        .font_family(roboto_font)
                        .font_size(48);
                });
        })
        .hover(|style| {
            style
                .display(DisplayLayout::InlineBlock)
                .background(|bg| {
                    bg.color(Color::GREEN);
                })
                .border(|br| {
                    br.radius(100);
                })
                .size(Size {
                    width: 150,
                    height: 100,
                })
                .transform(|transform| {
                    transform.scale_x(1.5).scale_y(1.25);
                });
        });

    scene.on_event::<WindowCloseRequest>(|context, _| {
        context.close(0);
    });

    prisma.add_window(
        WindowBuilder::new("Hello World!")
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