use prismae::Prisma;
use prismae::Scene;
use prismae::WindowBuilder;
use prismae::error::PrismaError;
use prismae::event::Click;
use prismae::event::NodeCreation;
use prismae::event::WindowCloseRequest;
use prismae::node::NodeID;
use prismae::resources::Font;
use prismae::util::Color;

type PopupActive = bool;
type PopupID = NodeID;

fn main() -> Result<(), PrismaError> {
    let mut prisma: Prisma = Prisma::init()?;

    let font_roboto = prisma.register::<Font>("examples/assets/font/Roboto.ttf")?;

    let mut scene = Scene::new();

    let button = scene
        .new_node()
        .with_style(move |style| {
            style
                .bg_color(Color::RED)
                .size(100, 100)
                .position(100, 100)
                .content("Abrir popup".to_string())
                .font_family(font_roboto.to_owned());
        })
        .on_hover(|style| {
            style.bg_color(Color::GREEN);
        })
        .on_event::<NodeCreation>(|ctx, _| {
            ctx.current_target()
                .unwrap()
                .set_state::<PopupActive>(false);
        })
        .on_event::<Click>(move |ctx, _| {
            let mut node = ctx.current_target().unwrap();
            let popup_active = *node.get_state::<PopupActive>().unwrap();
            node.set_state::<PopupActive>(!popup_active);

            if popup_active {
                let popup = node.remove_state::<PopupID>().unwrap();
                ctx.destroy(popup);
            } else {
                let popup = ctx
                    .new_node()
                    .with_style(move |style| {
                        style
                            .bg_color(Color::YELLOW)
                            .layer(1)
                            .position(200, 200)
                            .size(500, 350)
                            .content("chupalo".to_string())
                            .font_family(font_roboto);
                    })
                    .get_id();
                ctx.current_target().unwrap().set_state::<PopupID>(popup);
            }
        })
        .get_id();

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
