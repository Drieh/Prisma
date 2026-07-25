use prismae::Prisma;
use prismae::Scene;
use prismae::WindowBuilder;
use prismae::event::{
    Event, EventType, LifecycleEventType, MouseEvent, MouseEventType, WindowEventType,
};
use prismae::nodes::NodeAction;
use prismae::util::{Color, Position};

fn main() {
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
        .expect("Failed to create window.")
        .build()
        .expect("Failed to build Prisma.")
        .run();
}

pub fn example_scene() -> Scene {
    let mut scene = Scene::new();

    let _node_1 = scene
        .new_node()
        .layer(1)
        .position(100, 0)
        .bg_color(Color::rgb(255, 0, 0))
        .size(100, 100)
        .hover(&[NodeAction::BGColor {
            color: Color::rgb(150, 55, 55),
        }])
        .on_event(EventType::Lifecycle(LifecycleEventType::Creation), |ctx| {
            println!(
                "Node with id {} created, the red one. Hover me!",
                ctx.target().unwrap().get_id()
            );
            ctx.new_node()
                .bg_color(Color::rgb(0, 0, 200))
                .border_radius(100)
                .on_event(EventType::Lifecycle(LifecycleEventType::Creation), |ctx| {
                    println!(
                        "Hello, I'm node with id {}, the blue one!",
                        ctx.target().unwrap().get_id()
                    );
                    //ctx.current_target().unwrap().destroy();
                })
                .position_absolute()
                .scale(1.0, 1.0)
                .hover(&[NodeAction::Scale { x: 3.0, y: 2.0 }]);
        });

    let _node_2 = scene
        .new_node()
        .border_radius(25)
        .position(0, 300)
        .bg_color(Color::rgb(0, 200, 100))
        .size(100, 100)
        .active(&[NodeAction::BGColor {
            color: Color {
                r: 200,
                g: 200,
                b: 100,
                a: 200,
            },
        }])
        .on_event(EventType::Mouse(MouseEventType::DragStart), |ctx| {
            if let Event::Mouse {
                event: MouseEvent::DragStart { x, y, .. },
            } = ctx.event()
                && let Some(mut target) = ctx.target()
            {
                let Position {
                    x: node_x,
                    y: node_y,
                } = target.get_transform().position;
                target.set_state("offset", (x - node_x, y - node_y));
            }
        })
        .on_event(EventType::Mouse(MouseEventType::Drag), |ctx| {
            if let Event::Mouse {
                event: MouseEvent::Drag { x, y, .. },
            } = ctx.event()
                && let Some(mut target) = ctx.target()
            {
                let (node_x, node_y) = *target.get_state::<(f32, f32)>("offset").unwrap();
                target.position((x - node_x) as i32, (y - node_y) as i32);
            }
        });

    scene.bg_color(200, 200, 200);
    scene.on_event(EventType::Window(WindowEventType::CloseRequest), |ctx| {
        ctx.close(0);
    });

    scene
}
