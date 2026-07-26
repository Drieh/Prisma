use prismae::Prisma;
use prismae::Scene;
use prismae::WindowBuilder;
use prismae::error::PrismaError;
use prismae::event::EventType;
use prismae::event::WindowEventType;

use prismae::util::Color;

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
