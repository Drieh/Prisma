use prismae::Prisma;
use prismae::Scene;
use prismae::WindowBuilder;
use prismae::error::PrismaError;
use prismae::event::WindowCloseRequest;
use prismae::util::Color;

fn main() -> Result<(), PrismaError> {
    let mut prisma = Prisma::init()?;
    let mut scene = Scene::new();

    scene.new_node().style(|style| {
        style
            .background(|bg| {
                bg.color(Color::RED);
            })
            .border(|border| {
                border.radius(24);
            })
            .padding(|p| {
                p.top(12);
            })
            .text(|txt| {
                txt.content("hola");
            });
    });

    scene.on_event::<WindowCloseRequest>(|ctx, _| {
        ctx.close(0);
    });

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
