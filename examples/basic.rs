use prismae::Prisma;
use prismae::Scene;
use prismae::WindowBuilder;
use prismae::error::PrismaError;
use prismae::event::WindowCloseRequest;
use prismae::node::StyleView;
use prismae::util::Color;

fn main() -> Result<(), PrismaError> {
    let mut scene = Scene::new();

    scene
        .new_node()
        .bg_color(Color::ORANGE)
        .size(150, 150)
        .border_radius(25)
        .position(100, 100);

    scene.on_event::<WindowCloseRequest>(|context, _| {
        context.close(0);
    });

    let mut app = Prisma::init()?;
    app.add_window(
        WindowBuilder::new("Example")
            .position_centered()
            .resizable()
            .size(800, 400),
        scene,
    )?;
    app.run()?;
    Ok(())
}
