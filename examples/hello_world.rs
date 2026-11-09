use std::path::Path;
use std::path::PathBuf;

use prismae::Prisma;
use prismae::Scene;
use prismae::WindowBuilder;
use prismae::error::PrismaError;
use prismae::event::WindowCloseRequest;
use prismae::resources::Font;
use prismae::util::Color;

fn main() -> Result<(), PrismaError> {
    let mut prisma: Prisma = Prisma::init()?;

    let roboto_font = prisma.register::<Font>("examples/assets/font/Roboto.ttf")?;

    let mut scene = Scene::new();

    scene
        .new_node()
        .with_style(move |style| {
            style
                .bg_color(Color::LIME)
                .size(200, 200)
                .border_radius(25)
                .position(200, 100)
                .content("hola".to_string())
                .font_family(roboto_font)
                .font_size(24);
        })
        .on_hover(|style| {
            style.font_size(48);
        })
        .on_active(|style| {
            style.font_size(96);
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
