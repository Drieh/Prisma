use prisma::Prisma;
use prisma::WindowBuilder;
use prisma::event::Event;
use prisma::event::LifecycleEventType;
use prisma::event::MouseEvent;
use prisma::event::WindowEventType;
use prisma::scene::NodeID;
use prisma::scene::Scene;
use prisma::util::Position;

use prisma::scene::NodeAction;

use prisma::event::EventType;
use prisma::event::MouseEventType;
use sdl3::pixels::Color;

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
        .expect("Failed to create window 2.")
        .build()
        .expect("Failed to build Prisma.")
        .run();
}

pub fn example_scene() -> Scene {
    let mut scene = Scene::new();

    let _node_1 = scene
        .new_node()
        .position(100, 0)
        .bg_color(255, 0, 0, 255)
        .size(100, 100)
        .on_hover(&[NodeAction::BGColor {
            color: Color::RGB(150, 55, 55),
        }])
        .on_event(EventType::Lifecycle(LifecycleEventType::Creation), |ctx| {
            println!("node 1 created");
            ctx.get_node(NodeID::id(1)).unwrap().destroy();
            ctx.target().unwrap().destroy();
            ctx.new_node()
                .bg_color(0, 0, 255, 255)
                .border_radius(100)
                .on_event(EventType::Lifecycle(LifecycleEventType::Creation), |ctx| {
                    println!("hola soy node id 2");
                    //ctx.current_target().unwrap().destroy();
                })
                .position_absolute()
                .scale(1.0, 1.0)
                .on_hover(&[NodeAction::Scale { x: 3.0, y: 2.0 }]);
        });

    let _node_2 = scene
        .new_node()
        .border_radius(25)
        .position(0, 300)
        .bg_color(0, 200, 100, 255)
        .size(100, 100)
        .bg_color(255, 100, 100, 255)
        .on_active(&[NodeAction::BGColor {
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
                && let Some(mut target) = ctx.og_target()
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
                && let Some(mut target) = ctx.og_target()
            {
                let (node_x, node_y) = *target.get_state::<(f32, f32)>("offset").unwrap();

                target.position((x - node_x) as i32, (y - node_y) as i32);
            }
        });

    scene.bg_color(200, 200, 200);
    scene.on(EventType::Window(WindowEventType::CloseRequest), |ctx| {
        ctx.close(0);
    });
    // destruccion insegura corregir
    //scene.get_node(NodeID::id(1)).unwrap().destroy();
    scene
}
