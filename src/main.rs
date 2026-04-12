use std::collections::HashMap;
use std::sync::{Arc};

use clap::Parser;
use render_engine::ds::Vector3;
use render_engine::object::Renderable;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent, DeviceEvent, DeviceId};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId, CursorGrabMode};
use winit::keyboard::{KeyCode, PhysicalKey};
use render_engine::{ds, object, object::renderable::ToGpu, wgpu_handler};

mod gameobject;

use crate::gameobject::GameObject;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long, default_value_t = String::from("1280x720"))]
    pub resolution: String,

    #[arg(short, long, default_value_t = 0.001)]
    pub sensitivity: f64,

    #[arg(short, long, default_value_t = 144)]
    pub framelimit: u64,

    #[arg(short, long, default_value_t = 1.5)]
    pub movespeed: f64,

    #[arg(short = 'v', long, default_value_t = 60.0)]
    pub fov: f64,

    #[arg(default_value_t = 1920)]
    pub width: usize,
    #[arg(default_value_t = 1080)]
    pub height: usize,
}

impl Arguments {
    fn update(&mut self) {
        let mut split = self.resolution.split('x');
        let res_err = "Invalid resolution";
        (self.width, self.height) = (
            split.next().expect(res_err).parse().expect(res_err),
            split.next().expect(res_err).parse().expect(res_err)
        );

        if self.framelimit > 1000 {
            self.framelimit = 1000;
        }
    }
}

struct App {
    // window
    window: Option<Arc<Window>>,
    gpu:    wgpu_handler::GpuHandler,

    // scene
    player:  object::Player,
    gameobjects: HashMap<Vector3, Box<dyn GameObject>>,

    // input
    keyboard:     HashMap<KeyCode, bool>,
    mouse_delta: (f64, f64),
    config:       Arguments,

    // statistics
    last_frame: std::time::Instant,
    deltatime:  f64,

    fps_stat:         u32,
    deltatime_stat:   u32,
    statistics_timer: std::time::Instant,
}

impl App {
    pub fn new(config: Arguments, player: object::Player) -> Self {
        Self {
            window: None,
            gpu:    wgpu_handler::GpuHandler::default(),

            player:  player,
            gameobjects: HashMap::from([
                (Vector3::new( 0.0,  0.0, 3.0), Box::new(gameobject::Block::new_grass_block(&Vector3::new(0.0, 0.0, 3.0))) as Box<dyn GameObject>),
                (Vector3::new( 0.0, -1.0, 3.0), Box::new(gameobject::Block::new_dirt_block(&Vector3::new(0.0, -1.0, 3.0))) as Box<dyn GameObject>),
                (Vector3::new( 1.0, -1.0, 3.0), Box::new(gameobject::Block::new_grass_block(&Vector3::new(1.0, -1.0, 3.0))) as Box<dyn GameObject>),
                (Vector3::new(-1.0, -1.0, 3.0), Box::new(gameobject::Block::new_grass_block(&Vector3::new(-1.0, -1.0, 3.0))) as Box<dyn GameObject>)
            ]),


            keyboard:     HashMap::new(),
            mouse_delta: (0.0, 0.0),
            config:       config,

            last_frame: std::time::Instant::now(),
            deltatime:  0.0,

            fps_stat:         0,
            deltatime_stat:   0,
            statistics_timer: std::time::Instant::now(),
        }
    }

    pub fn handle_movement(&mut self) {
        // let mut player_ref = self.player();
        let key_movements: &[(KeyCode, ds::Vector3)] = &[
            (KeyCode::KeyW,        ds::Vector3::new( 0.0,  0.0,  1.0)),
            (KeyCode::KeyS,        ds::Vector3::new( 0.0,  0.0, -1.0)),
            (KeyCode::KeyA,        ds::Vector3::new(-1.0,  0.0,  0.0)),
            (KeyCode::KeyD,        ds::Vector3::new( 1.0,  0.0,  0.0)),
            (KeyCode::Space,       ds::Vector3::new( 0.0,  1.0,  0.0)),
            (KeyCode::ControlLeft, ds::Vector3::new( 0.0, -1.0,  0.0)),
        ];

        let key_rotations: &[(KeyCode, ds::Vector3)] = &[
            (KeyCode::ArrowLeft,  ds::Vector3::new( 0.0,  0.0, -0.5)),
            (KeyCode::ArrowRight, ds::Vector3::new( 0.0,  0.0,  0.5)),
            (KeyCode::ArrowUp,    ds::Vector3::new( 0.5,  0.0,  0.0)),
            (KeyCode::ArrowDown,  ds::Vector3::new(-0.5,  0.0,  0.0)),
        ];

        for (key, dir) in key_movements {
            if self.keyboard.get(key) == Some(&true) {
                self.player.move_player(&(dir * self.config.movespeed * self.deltatime));
            }
        }

        for (key, dir) in key_rotations {
            if self.keyboard.get(key) == Some(&true) {
                self.player.change_rotation(dir * self.deltatime);
            }
        }

        self.player.change_rotation(ds::Vector3::new(-self.mouse_delta.1, 0.0, self.mouse_delta.0));
        self.mouse_delta = (0.0, 0.0);

        self.player.update_outputs();
    }

    pub fn render(&self) -> Option<wgpu::SurfaceTexture> {

        // downcast objects
        let mut uniform = self.player.get_camera().to_gpu();

        let mut gpu_quads: Vec<object::quad::GpuQuad> = vec![];
        
        for (pos, go) in &self.gameobjects {
            let quads = go.get_renderables();

            for quad in quads {
                let facing_block_pos = 2.0*(quad.center()-pos) + pos;
                if self.gameobjects.contains_key(&facing_block_pos) {
                    continue;
                }
                
                let q = quad.as_any().downcast_ref::<object::Quad>().unwrap();
                gpu_quads.push(q.to_gpu());
            }
        }

        return self.gpu.draw_frame(&vec![], &gpu_quads, &mut uniform);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("Minceraft")
                    .with_inner_size(winit::dpi::LogicalSize::new(self.config.width as f64, self.config.height as f64))
            ).unwrap()
        );

        window.set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
            .unwrap();
        window.set_cursor_visible(false);

        // wgpu init is async but resumed() isn't — use pollster to block
        pollster::block_on(
            self.gpu.init(
                window.clone(), 
                self.config.width as u32, 
                self.config.height as u32, 
            vec!["textures/dirt.png", "textures/grass_side.png", "textures/grass_top.png"])
        );

        std::thread::sleep(std::time::Duration::from_millis(1000));

        self.window = Some(window);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                self.mouse_delta = (self.mouse_delta.0 + (dx as f64)*self.config.sensitivity, self.mouse_delta.1 + (dy as f64)*self.config.sensitivity);
            },
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                self.deltatime = self.last_frame.elapsed().as_millis() as f64 / 1000.0;
                self.last_frame = std::time::Instant::now();
                
                if self.window.is_none() {
                    println!("No Window");
                    return;
                }
                
                let window = self.window.as_ref().unwrap();
                let size = window.inner_size();
                
                self.player.get_camera_mut().set_window_size(size.width.into(), size.height.into());
                self.handle_movement();
                
                if let Some(frame) = self.render() {
                    frame.present();
                }
                
                // let player = self.player.read().unwrap();
                
                // if self.statistics_timer.elapsed().as_millis() >= 1000 {
                //     self.fps_stat = (1.0/self.deltatime) as u32;
                //     self.deltatime_stat = (1000.0 * self.deltatime) as u32;
                //     self.statistics_timer = std::time::Instant::now();
                // }
                
                // print!("\x1B[2J\x1B[1;1H");
                // println!(" FPS: {}\n\n Time between frames: {}ms\n\n Camera position: {:?}\n Player Rotation: {:?}", self.fps_stat, self.deltatime_stat, player.get_camera().pos(), player.get_rotation());

                // this makes the deltatime not crash out when the fps gets too high,
                // but caps the fps at 1000
                if self.deltatime < 1000.0/self.config.framelimit as f64 {
                    let mut duration = std::time::Duration::from_millis(1000/self.config.framelimit);
                    duration -= std::time::Duration::from_millis(self.deltatime as u64);
                    std::thread::sleep(duration);
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::Resized(new_size) => {
                let window = match &self.window {
                    Some(t) => t,
                    None => return,
                };

                self.gpu.change_resolution(new_size.width.into(), new_size.height.into());
                
                window.request_redraw();
            }

            WindowEvent::KeyboardInput{
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    state,
                    ..
                },
                ..
            } => {
                match (keycode, state) {
                    // special functions
                    (KeyCode::Escape, ElementState::Pressed) => event_loop.exit(),
                    (KeyCode::KeyE, ElementState::Pressed) => {
                        let ray = ds::Ray::new(&self.player.get_camera().pos(), &(self.player.get_camera().dir() - self.player.get_camera().pos()));

                        let (t, renderable, go_pos) = match get_renderable_looking_at(&ray, &self.gameobjects) {
                            Some(t) => t,
                            None => return
                        };

                        let new_center = go_pos - renderable.normal(&ray.at(t));
                        
                        self.gameobjects.insert(new_center, Box::new(gameobject::Block::new_dirt_block(&new_center)));

                    },
                    (KeyCode::KeyQ, ElementState::Pressed) => {
                        let ray = ds::Ray::new(&self.player.get_camera().pos(), &(self.player.get_camera().dir() - self.player.get_camera().pos()));

                        let (_t, _renderable, go_pos) = match get_renderable_looking_at(&ray, &self.gameobjects) {
                            Some(t) => t,
                            None => return
                        };

                        self.gameobjects.remove(&go_pos);
                    }
                    (keycode, pressed) => {
                        // everything else is mapped to the keyboard hashmap
                        self.keyboard.insert(keycode, pressed == ElementState::Pressed);
                    }
                }
            }

            _ => {}
        }
    }
}

fn get_renderable_looking_at<'a>(ray: &ds::Ray, gameobjects: &'a HashMap<Vector3, Box<dyn GameObject>>) -> Option<(f64, &'a dyn Renderable, Vector3)> {
    let mut closest_renderable: Option<(f64, &dyn Renderable)> = None;
    let mut closest_pos: Vector3 = Vector3::zero();

    for (pos, go) in gameobjects {
        // println!("Checking block at {:?}", pos);
        let intersection = go.intersects(&ray);

        if intersection.is_none() {
            // println!("  No Intersection");
            continue;
        }

        if closest_renderable.is_none() {
            closest_renderable = intersection;
            closest_pos = pos.clone();
        } else {
            if intersection.unwrap().0 < closest_renderable.unwrap().0 {
                closest_renderable = intersection;
                closest_pos = pos.clone();
            }
        }
    }

    if closest_renderable.is_none() {
        return None;
    }

    return Some((
        closest_renderable.unwrap().0,
        closest_renderable.unwrap().1,
        closest_pos
    ))
}

fn main() {
    debug_assert_eq!(std::mem::size_of::<object::camera::GpuUniform>() % 256, 0);

    let mut args = Arguments::parse();
    args.update();

    let camera = object::Camera::new(
        ds::Vector3::new(0.0, 0.0, 0.0),
        3.0,
        (args.width as f64, args.height as f64),
        args.fov
    );

    let player = object::Player::new(
        camera
    );

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(args, player);

    event_loop.run_app(&mut app).expect("Event loop failed");
}