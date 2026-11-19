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
use prismae::util::DisplayLayout;
use prismae::util::Size;

type PopupActive = bool;
type PopupID = NodeID;

fn main() -> Result<(), PrismaError> {
    let mut prisma: Prisma = Prisma::init()?;

    let font_roboto = prisma.register::<Font>("examples/assets/font/Roboto.ttf")?;

    let mut scene = Scene::new();

    scene
        .new_node()
        .style(move |style| {
            style
                .background(|bg| {
                    bg.color(Color::GREEN);
                })
                .size(Size {
                    width: 100,
                    height: 100,
                })
                .transform(|transform| {
                    transform.x(100).y(100);
                })
                .text(move |text| {
                    text.content("abrir popup!".into())
                        .font_family(font_roboto)
                        .font_size(32);
                });
        })
        .hover(|style| {
            style
                .background(|bg| {
                    bg.color(Color::GREEN);
                })
                .text(|text| {
                    //text.content("abrir popup hover".into());
                });
        })
        .on_event::<NodeCreation>(move |ctx, _| {
            ctx.current_target().set_state::<PopupActive>(false);
            let popup = ctx
                .new_node()
                .style(move |style| {
                    style
                        .background(|bg| {
                            bg.color(Color::YELLOW);
                        })
                        .transform(|transform| {
                            transform.layer(1).x(200).y(200);
                        })
                        .size(Size {
                            width: 200,
                            height: 200,
                        })
                        .text(move |text| {
                            text.content("hola".into()).font_family(font_roboto);
                        })
                        .display(DisplayLayout::None);
                })
                .hover(|style| {
                    style.display(DisplayLayout::Block);
                })
                .get_id();
            ctx.current_target().set_state::<PopupID>(popup);
        })
        .on_event::<Click>(|ctx, _| {
            let mut node = ctx.current_target();
            let popup_active = *node.get_state::<PopupActive>().unwrap();
            let popup = *node.get_state::<PopupID>().unwrap();

            node.set_state::<PopupActive>(!popup_active);
            if popup_active {
                ctx.get_node(popup).unwrap().style(|style| {
                    style.display(DisplayLayout::None);
                });
            } else {
                ctx.get_node(popup).unwrap().style(|style| {
                    style.display(DisplayLayout::InlineBlock);
                });
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
