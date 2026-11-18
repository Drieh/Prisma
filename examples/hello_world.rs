use prismae::Prisma;
use prismae::Scene;
use prismae::WindowBuilder;
use prismae::error::PrismaError;
use prismae::event::Click;
use prismae::event::WindowCloseRequest;
use prismae::resources::Font;
use prismae::util::Color;
use prismae::util::DisplayLayout;

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
                    br.radius(24);
                })
                .text(move |txt| {
                    txt.content("hola".into())
                        .font_family(roboto_font)
                        .font_size(12);
                })
                .display(DisplayLayout::InlineBlock);
        })
        .hover(|style| {
            style
                .text(|txt| {
                    txt.font_size(64);
                })
                .background(|bg| {
                    bg.color(Color::RED);
                });
            println!("hover");
        })
        .active(|style| {
            style.text(|txt| {
                txt.font_size(96);
            });
        })
        .on_event::<Click>(|ctx, _| {
            ctx.current_target().style(|style| {
                style.background(|bg| {
                    bg.color(Color::GREEN);
                });
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
