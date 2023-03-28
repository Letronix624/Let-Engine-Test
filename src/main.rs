use let_engine::*;
use winit::dpi::LogicalSize;
use winit::event::Event;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;
use winit::window::WindowBuilder;
fn main() {
    //yes:
    let app_info = AppInfo {
        app_name: "Let Engine Test",
    };
    let window_builder = WindowBuilder::new()
        .with_resizable(false)
        .with_title("Test Window")
        .with_min_inner_size(LogicalSize::new(150, 150))
        .with_inner_size(LogicalSize::new(1000, 1000))
        .with_always_on_top(false)
        .with_decorations(true)
        .with_transparent(true)
        .with_visible(false);
    let (mut game, event_loop) = GameBuilder::new()
        .with_window_builder(window_builder)
        .with_app_info(app_info)
        .build();
    game.load_font_bytes(
        "Regular",
        include_bytes!("../assets/fonts/Hand-Regular.ttf"),
    );
    let layer = game.new_layer();
    let mut txt = String::from("Hello there tester!");
    let fsize = 35.0;
    let rtext = Object::new().graphics(Some(
        Appearance::new_color([1.0, 0.0, 0.0, 1.0]).size([1.0, 0.5]),
    ));
    let gtext = Object::new().graphics(Some(
        Appearance::new_color([0.0, 1.0, 0.0, 1.0]).size([1.0, 0.5]),
    ));
    let btext = Object::new().graphics(Some(
        Appearance::new_color([0.0, 0.0, 1.0, 1.0]).size([1.0, 0.5]),
    ));

    let rtext = game.add_object(&layer, rtext).unwrap();
    let gtext = game.add_object(&layer, gtext).unwrap();
    let btext = game.add_object(&layer, btext).unwrap();

    game.label(&rtext, "Regular", &txt, fsize, NW);
    game.label(&gtext, "Regular", &txt, fsize, CENTER);
    game.label(&btext, "Regular", &txt, fsize, SO);

    let camera = game.add_object(&layer, Object::new()).unwrap();
    camera.lock().camera = Some(CameraOption {
        zoom: 1.0,
        mode: CameraScaling::Expand,
    });
    game.set_camera(&layer, &camera).unwrap();
    game.get_window().set_visible(true);
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(_) => game.recreate_swapchain(),
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                        control_flow.set_exit();
                    }
                }
                WindowEvent::ReceivedCharacter(c) => {
                    match c {
                        '\u{8}' => {
                            txt.pop();
                        }
                        _ if c != '\u{7f}' => txt.push(c),
                        _ => {}
                    }
                    game.label(&rtext, "Regular", &txt, fsize, NW);
                    game.label(&gtext, "Regular", &txt, fsize, CENTER);
                    game.label(&btext, "Regular", &txt, fsize, SO);
                }
                _ => (),
            },
            Event::RedrawEventsCleared => {
                game.update();
            }
            _ => (),
        }
    });
}
