use prisma::Position;
use prisma::Prisma;
use prisma::WindowBuilder;
use prisma::event::Event;
use prisma::event::LifecycleEventType;
use prisma::event::MouseEvent;
use prisma::event::WindowEventType;
use prisma::scene::Scene;

use prisma::scene::NodeAction;

use prisma::event::EventType;
use prisma::event::MouseEventType;
use sdl3::pixels::Color;

fn main() {
    /* HIT TEST meedio aun ROTOOOO */
    let scene = example_scene();

    let _builder = Prisma::builder()
        .expect("Failed to create app builder.")
        .window(
            WindowBuilder::new("Example")
                .position_centered()
                .resizable()
                .size(800, 400),
            scene,
        )
        .expect("Failed to create window 2.")
        .build()
        .expect("Failed to build Prisma.")
        .run();
}

pub fn example_scene() -> Scene {
    let mut scene = Scene::new();

    let node_1 = scene
        .new_node()
        .position(0, 0)
        .bg_color(255, 0, 0, 255)
        .size(100, 100)
        .on_hover(&[
            NodeAction::BGColor {
                color: Color {
                    r: 200,
                    g: 200,
                    b: 100,
                    a: 50,
                },
            },
            NodeAction::Position { x: 50, y: 50 },
        ])
        .id();
    let node_2 = scene
        .new_node()
        .position(0, 300)
        .bg_color(0, 200, 100, 255)
        .size(100, 100)
        .on_active(&[NodeAction::BGColor {
            color: Color {
                r: 200,
                g: 200,
                b: 100,
                a: 50,
            },
        }])
        .on_event(
            EventType::Lifecycle(LifecycleEventType::Creation),
            Box::new(|ctx, tg| {
                tg.set_state("offset", (0, 0));
            }),
        )
        .on_event(
            EventType::Mouse(MouseEventType::DragStart),
            Box::new(|ctx, tg| {
                if let Some(Event::Mouse {
                    event: MouseEvent::DragStart { x, y, .. },
                }) = ctx.event
                {
                    let Position {
                        x: node_x,
                        y: node_y,
                    } = tg.get_transform().position;

                    tg.set_state("offset", (x - node_x, y - node_y));
                }
            }),
        )
        .on_event(
            EventType::Mouse(MouseEventType::Drag),
            Box::new(|ctx, tg| {
                if let Some(Event::Mouse {
                    event: MouseEvent::Drag { x, y, .. },
                }) = ctx.event
                {
                    let (node_x, node_y) = tg.get_state::<(f32, f32)>("offset").unwrap();

                    tg.position((x - node_x) as i32, (y - node_y) as i32);
                }
            }),
        )
        .id();

    scene.bg_color(200, 200, 200);
    scene.on(
        EventType::Window(WindowEventType::CloseRequest),
        Box::new(|context| {
            context.close(10);
        }),
    );

    scene
}
