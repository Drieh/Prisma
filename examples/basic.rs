use prisma::Prisma;
use prisma::Scene;
use prisma::WindowBuilder;

use prisma::event::Event;
use prisma::event::EventType;
use prisma::event::LifecycleEventType;
use prisma::event::MouseButton;
use prisma::event::MouseEvent;
use prisma::event::MouseEventType;
use prisma::event::WindowEventType;

fn main() {
    let scene_1 = example_scene();
    let scene_2 = example_scene();

    let _builder = Prisma::builder()
        .expect("Failed to create app builder.")
        .window(
            WindowBuilder::new("Example")
                .position(500, 500)
                .resizable()
                .size(500, 500),
            scene_1,
        )
        .expect("Failed to create window 1.")
        .window(
            WindowBuilder::new("Example 2")
                .position_centered()
                .resizable()
                .size(500, 500),
            scene_2,
        )
        .expect("Failed to create window 2.")
        .build()
        .expect("Failed to create Prisma")
        .run();
}

pub fn example_scene() -> Scene {
    let mut scene = Scene::new();

    let node_1 = scene.new_node();
    let node_2 = scene.new_node();
    let node_3 = scene.new_node();

    scene.bg_color(200, 200, 200);

    scene.on_scene(
        EventType::Window(WindowEventType::CloseRequest),
        Box::new(|context| {
            context.close(10);
        }),
    );

    scene
        .node(node_1)
        .unwrap()
        .position(0, 0)
        .bg_color(255, 0, 0, 255)
        .size(100, 100);

    scene
        .node(node_2)
        .unwrap()
        .position(100, 100)
        .bg_color(0, 255, 0, 255)
        .size(100, 100);

    scene
        .node(node_3)
        .unwrap()
        .position(100, 100)
        .bg_color(0, 0, 255, 255)
        .size(100, 100);

    scene.add_child(node_2, node_3).expect("msg");

    scene.on_node(
        node_1,
        EventType::Mouse(MouseEventType::Click),
        Box::new(|context, target| {
            println!("Closing in 3 sec. Byee, att node 1.");
            context.close(500);
            target
                .bg_color(100, 0, 0, 255)
                .wait(300)
                .bg_color(255, 0, 0, 255);
        }),
    );
    scene.on_node(
        node_2,
        EventType::Lifecycle(LifecycleEventType::Creation),
        Box::new(|_context, target| {
            target.set_state("clicks", 0);
        }),
    );
    scene.on_node(
        node_2,
        EventType::Mouse(MouseEventType::Click),
        Box::new(|context, target| {
            if let Some(Event::Mouse {
                event:
                    MouseEvent::Click {
                        mouse_btn: MouseButton::Left,
                        ..
                    },
            }) = context.event
            {
                if let Some(clicks) = target.get_state_mut::<i32>("clicks") {
                    *clicks += 1;
                    println!("node 2 clicks count: {clicks}");
                }
            }
        }),
    );
    scene
}
