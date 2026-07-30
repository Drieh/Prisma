use prismae::{
    Prisma, Scene, WindowBuilder,
    error::PrismaError,
    event::{Drag, DragStart, MouseDown, MouseUp, WindowCloseRequest},
    util::{Color, Position},
};

fn main() -> Result<(), PrismaError> {
    let scene = drag_example_scene();

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

fn drag_example_scene() -> Scene {
    let mut scene = Scene::new();

    scene.on_event::<WindowCloseRequest>(|ctx, _| {
        ctx.close(0);
    });

    scene
        .new_node()
        .position(100, 100)
        .bg_color(Color::rgb(200, 200, 50))
        .border_radius(10)
        .size(150, 150)
        .on_active(|style| {
            style.bg_color(Color::rgb(255, 0, 0)).size(100, 100);
        })
        .on_hover(|style| {
            style.bg_color(Color::rgb(100, 100, 200));
        })
        .on_event::<MouseDown>(|ctx, _| {
            if let Some(mut target) = ctx.current_target() {
                let Position { x, y } = target.get_relative_position();
                target.position(x as i32 + 25, y as i32 + 25);
            }
        })
        .on_event::<MouseUp>(|ctx, _| {
            if let Some(mut target) = ctx.current_target() {
                let Position { x, y } = target.get_relative_position();
                target.position(x as i32 - 25, y as i32 - 25);
            }
        })
        .on_event::<DragStart>(|ctx, event| {
            if let Some(mut target) = ctx.current_target() {
                let Position {
                    x: node_x,
                    y: node_y,
                } = target.get_relative_position();
                let offset = (event.position.x - node_x, event.position.y - node_y);

                target.set_state("offset", offset);
            }
        })
        .on_event::<Drag>(|ctx, event| {
            if let Some(mut target) = ctx.current_target() {
                let (offset_x, offset_y) = target.get_state::<(f32, f32)>("offset").unwrap();
                target.position(
                    (event.position.x - offset_x) as i32,
                    (event.position.y - offset_y) as i32,
                );
            }
        });

    scene
}
