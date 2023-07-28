use let_engine::physics::*;
use let_engine::prelude::*;
use spin_sleep::sleep;
use std::{
    thread,
    time::{Duration, SystemTime},
};
use winit::dpi::PhysicalSize;
use winit::event::{
    ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use winit::window::{Fullscreen, WindowBuilder};

const TICK_SPEED: f32 = 180.0;

fn main() {
    let window_builder = WindowBuilder::new()
        .with_resizable(true)
        .with_title("Test Window")
        .with_min_inner_size(PhysicalSize::new(150.0, 150.0))
        .with_inner_size(PhysicalSize::new(1000.0, 700.0))
        .with_decorations(true)
        .with_visible(false);
    let (mut game, event_loop) = GameBuilder::new()
        .with_window_builder(window_builder)
        .build();

    let resources = game.resources.clone();
    let input = game.input.clone();
    let scene = game.scene.clone();

    let font = resources.load_font(include_bytes!("../assets/fonts/Px437_CL_Stingray_8x16.ttf"));
    let layer = game.scene.new_layer();
    let mut txt = String::from("Left mouse button: spawn object\rRight mouse button: remove object\rMiddle mouse: Zoom and pan\rEdit this text with the keyboard.");
    let fsize = 35.0;
    let mut rtext = Label::new(
        &resources,
        &font,
        LabelCreateInfo {
            appearance: Appearance::new_color([1.0, 0.0, 0.0, 1.0])
                .transform(Transform::default().size(vec2(2.0, 2.0))),
            text: txt.clone(),
            scale: vec2(fsize, fsize),
            align: NW,
            ..Default::default()
        },
    );
    let mut gtext = Label::new(
        &resources,
        &font,
        LabelCreateInfo {
            appearance: Appearance::new_color([0.0, 1.0, 0.0, 1.0])
                .transform(Transform::default().size(vec2(2.0, 2.0))),
            text: txt.clone(),
            scale: vec2(fsize, fsize),
            align: CENTER,
            ..Default::default()
        },
    );
    let mut btext = Label::new(
        &resources,
        &font,
        LabelCreateInfo {
            appearance: Appearance::new_color([0.0, 0.0, 1.0, 1.0])
                .transform(Transform::default().size(vec2(2.0, 2.0))),
            text: txt.clone(),
            scale: vec2(fsize, fsize),
            align: SO,
            ..Default::default()
        },
    );
    layer.add_object(None, &mut rtext).unwrap();
    layer.add_object(None, &mut gtext).unwrap();
    layer.add_object(None, &mut btext).unwrap();
    let mut camera = Camera::default();
    camera.camera.mode = CameraScaling::Expand;
    layer.add_object(None, &mut camera).unwrap();
    game.set_clear_background_color([0.35, 0.3, 0.31, 1.0]);
    layer.set_camera(camera.clone());

    resources.get_window().set_visible(true);
    resources.get_window().set_fullscreen(None);

    let place_indicator_material = resources.new_material(
        materials::MaterialSettingsBuilder::default()
            .topology(materials::Topology::LineStrip)
            .line_width(2.0)
            .build()
            .unwrap(),
    );

    let mut place_indicator = Object::default();
    place_indicator.appearance = Appearance::new()
        .material(place_indicator_material)
        .data(Data {
            vertices: vec![
                Vertex {
                    position: vec2(-1.0, -1.0),
                    tex_position: vec2(-1.0, -1.0),
                }, 
                Vertex {
                    position: vec2(1.0, -1.0),
                    tex_position: vec2(1.0, -1.0),
                }, 
                Vertex {
                    position: vec2(1.0, 1.0),
                    tex_position: vec2(1.0, 1.0),
                }, 
                Vertex {
                    position: vec2(-1.0, 1.0),
                    tex_position: vec2(-1.0, 1.0),
                }, 
            ],   
            indices: vec![0, 1, 2, 3, 0]
        });

    layer.add_object(None, &mut place_indicator).unwrap();

    let mut platform = ColliderObject::default();
    platform.appearance = Appearance::new()
        .data(Data::square())
        .color([0.9, 0.9, 0.9, 1.0]);
    platform.transform.size = vec2(5.0, 0.1);
    platform.transform.position = layer.side_to_world(S, (1000.0, 700.0));

    platform.set_collider(Some(
        ColliderBuilder::square(5.0, 0.1).restitution(0.0).build(),
    ));
    platform.set_rigid_body(Some(RigidBodyBuilder::fixed().build()));

    layer.add_object(None, &mut platform).unwrap();

    let mut last = false;
    let mut last2 = false;
    let mut right = false;
    let mut mouselock = vec2(0.0, 0.0);
    let mut camera_lock = vec2(0.0, 0.0);
    let mut time_scale: f32 = 1.0;
    let mut egui_focused = false;
    let mut physics_params = IntegrationParameters {
        dt: 1.0 / TICK_SPEED,
        max_stabilization_iterations: 3,
        allowed_linear_error: 0.0001,
        prediction_distance: 0.001,
        ..Default::default()
    };
    layer.set_physics_parameters(physics_params);

    let mut fixed = false;

    let mut color: [f32; 4] = [0.874509804, 0.082352941, 0.082352941, 1.0]; // default color

    let mut object_transform: Transform = (vec2(0.0, 0.0), vec2(0.07, 0.07), 0.0).into();
    let mut rotation: f32 = 0.0;

    let _tick_system = thread::spawn(|| tick_system(scene));

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match &event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                        control_flow.set_exit();
                    } else if input.virtual_keycode == Some(VirtualKeyCode::F11)
                        && input.state == ElementState::Released
                    {
                        let window = resources.get_window();
                        if window.fullscreen().is_some() {
                            window.set_fullscreen(None);
                        } else {
                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        }
                    }
                }
                WindowEvent::ReceivedCharacter(c) => {
                    if egui_focused {
                        return
                    };
                    match c {
                        '\u{8}' => {
                            txt.pop();
                        }
                        _ if *c != '\u{7f}' => txt.push(*c),
                        _ => {}
                    }
                    rtext.update_text(txt.clone());
                    gtext.update_text(txt.clone());
                    btext.update_text(txt.clone());
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if let MouseScrollDelta::LineDelta(_, y) = delta {
                        let zoom = camera.camera.zoom;
                        camera.camera.zoom = zoom + *y as f32 * zoom * 0.1;
                    };
                }
                _ => (),
            },
            Event::MainEventsCleared => {
                game.update_gui(|ctx| {
                    egui::TopBottomPanel::top("test").show(&ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut fixed, "Anchored");
                            let response = ui.add(
                                egui::Slider::new(&mut time_scale, 0.0..=1.0).text("Time scale"),
                            );
                            if response.changed() {
                                physics_params.dt = (1.0 / TICK_SPEED) * time_scale;
                                layer.set_physics_parameters(physics_params);
                            }
                            ui.add(egui::Slider::new(&mut object_transform.size.x, 0.01..=1.0).text("Size X"));
                            ui.add(egui::Slider::new(&mut object_transform.size.y, 0.01..=1.0).text("Size Y"));
                            ui.add(egui::Slider::new(&mut rotation, 0.0..=90.0).text("Rotation"));
                            object_transform.rotation = rotation.to_radians();
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut color[0], 0.0..=1.0).text("Red"));
                            ui.add(egui::Slider::new(&mut color[1], 0.0..=1.0).text("Green"));
                            ui.add(egui::Slider::new(&mut color[2], 0.0..=1.0).text("Blue"));
                            ui.add(egui::Slider::new(&mut color[3], 0.0..=1.0).text("Alpha"));
                        });
                        
                    });
                    egui_focused = ctx.is_pointer_over_area() || ctx.is_using_pointer() || ctx.wants_keyboard_input();
                });
                if egui_focused {
                    return;
                }

                object_transform.position = input.cursor_to_world(&layer);
                place_indicator.appearance.color = color;
                place_indicator.transform = object_transform;
                place_indicator.sync();
                
                {
                    if input.mouse_down(&MouseButton::Left) && !last {
                        let mut object = ColliderObject::default();
                        object.set_collider(Some(
                            ColliderBuilder::square(object_transform.size.x, object_transform.size.y) //trimesh(shape_data.clone())
                                .restitution(0.0)
                                .restitution_combine_rule(CoefficientCombineRule::Min)
                                .build(),
                        ));
                        let rigid_body_type = if fixed {
                            RigidBodyType::Fixed
                        } else {
                            RigidBodyType::Dynamic
                        };
                        object.set_rigid_body(Some(RigidBodyBuilder::new(rigid_body_type).build()));
                        object.appearance = Appearance::new().data(Data::square()).color(color);
                        object.transform = object_transform;
                        layer.add_object(None, &mut object).unwrap();
                    }
                    last = input.mouse_down(&MouseButton::Left);

                    if input.mouse_down(&MouseButton::Right) && !last2 {
                        let id = layer.cast_ray(
                            input.cursor_to_world(&layer),
                            vec2(0.0, 0.0),
                            0.0,
                            true,
                        );
                        if let Some(id) = id {
                            layer.remove_object(id).unwrap();
                        }
                    }
                    last2 = input.mouse_down(&MouseButton::Right);
                }
                {
                    let cp = input.scaled_cursor(&layer);
                    if input.mouse_down(&MouseButton::Middle) && !right {
                        mouselock = cp;
                        camera_lock = camera.transform.position;
                    }
                    if input.mouse_down(&MouseButton::Middle) {
                        let zoom = camera.camera.zoom;
                        let shift = vec2(
                            (mouselock[0] - cp[0]) * (1.0 / zoom) * 0.5 + camera_lock[0],
                            (mouselock[1] - cp[1]) * (1.0 / zoom) * 0.5 + camera_lock[1],
                        );
                        //times camera mode please
                        camera.transform.position = shift;
                    }
                    right = input.mouse_down(&MouseButton::Middle);
                }
                layer.set_camera(camera.clone());
            }
            _ => (),
        }
        game.update(&event);
    });
}

fn tick_system(scene: Scene) {
    let target_duration = Duration::from_secs_f32(1.0 / TICK_SPEED);
    loop {
        let start_time = SystemTime::now();
        tick(&scene);
        let elapsed_time = start_time.elapsed().unwrap();

        let waiting_time = target_duration.saturating_sub(elapsed_time);

        sleep(waiting_time);
    }
}

fn tick(scene: &Scene) {
    scene.iterate_all_physics();
}
