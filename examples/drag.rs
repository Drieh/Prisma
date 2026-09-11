use prismae::{
    Prisma, Scene, WindowBuilder,
    error::PrismaError,
    event::{Drag, DragEnd, DragStart, MouseDown, MouseUp, WindowCloseRequest},
    util::{Color, Position},
};
struct DragOffset {
    x: i32,
    y: i32,
}
fn main() -> Result<(), PrismaError> {
    let scene = drag_example_scene();

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

fn drag_example_scene() -> Scene {
    let mut scene = Scene::new();

    scene.on_event::<WindowCloseRequest>(|ctx, _| {
        ctx.close(0);
    });

    let mut node = scene.new_node();
    node.position(100, 100)
        .bg_color(Color::YELLOW)
        .border_radius(30)
        .size(150, 150);

    node.on_hover(|style| {
        style.bg_color(Color::rgb(100, 100, 200));
    })
    .on_active(|style| {
        style.bg_color(Color::RED).scale(0.67, 0.67);
    })
    .on_event::<MouseDown>(|ctx, _| {
        if let Some(mut target) = ctx.current_target() {
            let Position { x, y } = target.get_relative_position();
            target.position(x + 25, y + 25);
        }
    })
    .on_event::<MouseUp>(|ctx, _| {
        if let Some(mut target) = ctx.current_target() {
            let Position { x, y } = target.get_relative_position();
            target.position(x - 25, y - 25);
        }
    })
    .on_event::<DragStart>(|ctx, event| {
        if let Some(mut target) = ctx.current_target() {
            let Position {
                x: node_x,
                y: node_y,
            } = target.get_relative_position();
            let offset = DragOffset {
                x: event.position.x - node_x,
                y: event.position.y - node_y,
            };
            target.set_state::<DragOffset>(offset);
        }
    })
    .on_event::<Drag>(|ctx, event| {
        if let Some(mut target) = ctx.current_target() {
            let DragOffset { x, y } = target
                .get_state::<DragOffset>()
                .expect("Invariant violated: DragOffset state should be set on DragStart");
            target.position(event.position.x - x, event.position.y - y);
        }
    })
    .on_event::<DragEnd>(|ctx, _| {
        if let Some(mut target) = ctx.current_target() {
            target
                .remove_state::<DragOffset>()
                .expect("Invariant violated: DragOffset should be set on DragStart");
        }
    });
    scene
}
