use std::collections::HashMap;
use std::sync::{Arc};

use image::ImageReader;
use clap::{Parser};
use render_engine::ds::Vector3;
use render_engine::ui::ui_element::GPUUIElement;
use render_engine::ui;
use render_engine::wgpu_handler::GpuConfig;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
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
}
struct Config {
    pub sensitivity: f64,
    pub framelimit: u64,
    pub movespeed: f64,
    pub fov: f64,
    pub width: usize,
    pub height: usize
}

impl From<Arguments> for Config {
    fn from(a: Arguments) -> Self {
        let mut split = a.resolution.split('x');
        let res_err = "Invalid resolution";
        let (width, height) = (
            split.next().expect(res_err).parse().expect(res_err),
            split.next().expect(res_err).parse().expect(res_err)
        );

        let framelimit = a.framelimit.min(1000);

        Self {
            sensitivity: a.sensitivity,
            framelimit: framelimit,
            movespeed: a.movespeed,
            fov: a.fov,
            width: width,
            height: height
        }
    }
}

struct App {
    // window
    window: Option<Arc<Window>>,
    gpu:    wgpu_handler::GpuHandler,
    ui:     Vec<ui::UIElement>,

    // scene
    player:  gameobject::Player,
    gameobjects: HashMap<Vector3, Box<dyn GameObject>>,

    // input
    keyboard:     HashMap<KeyCode, bool>,
    mouse_delta: (f64, f64),
    config:       Config,

    // statistics
    last_frame: std::time::Instant,
    deltatime:  f64,
}

impl App {
    pub fn new(config: Config, player: gameobject::Player) -> Self {
        let mut this = Self {
            window: None,
            gpu:    wgpu_handler::GpuHandler::default(),
            ui:     vec![
                ui::UIElement::new(
                    ui::Image::new(
                        ImageReader::open("./textures/crosshair.png").expect("No image").decode().expect("Bad decode").to_rgba8().into_raw(),
                        16, 16
                    ).unwrap(),
                    ui::VerticalAnchor::Middle,
                    ui::HorizontalAnchor::Center
                )
            ],

            player:  player,
            gameobjects: HashMap::from([]),


            keyboard:     HashMap::new(),
            mouse_delta: (0.0, 0.0),
            config:       config,

            last_frame: std::time::Instant::now(),
            deltatime:  0.0,
        };

        for x in -10..10 {
            for z in -10..10 {
                this.gameobjects.extend(vec![
                    (Vector3::new( x as f64,  3.0, z as f64), Box::new(gameobject::Block::new_grass_block(&Vector3::new(x as f64,  3.0, z as f64))) as Box<dyn GameObject>),
                    (Vector3::new( x as f64,  2.0, z as f64), Box::new(gameobject::Block::new_dirt_block(&Vector3::new( x as f64,  2.0, z as f64))) as Box<dyn GameObject>),
                    (Vector3::new( x as f64,  1.0, z as f64), Box::new(gameobject::Block::new_dirt_block(&Vector3::new( x as f64,  1.0, z as f64))) as Box<dyn GameObject>),
                ]);
                
            }
        }

        this
    }

    pub fn handle_movement(&mut self) {
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

        let mut gpu_ui_elements: Vec<GPUUIElement> = vec![];
        for element in &self.ui {
            gpu_ui_elements.push(element.to_gpu());
        }

        return self.gpu.draw_frame(&vec![], &gpu_quads, &gpu_ui_elements, &mut uniform);
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
                &mut self.ui,
                vec!["textures/dirt.png", "textures/grass_side.png", "textures/grass_top.png"],
                GpuConfig {
                    quad_buffer_max: 100*100*3*6,
                    sphere_buffer_max: 48
                }
            )
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
            DeviceEvent::Button { button, state } => {
                println!("{:?}: {:?}", button, state);
            }
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
                    (keycode, pressed) => {
                        // everything else is mapped to the keyboard hashmap
                        self.keyboard.insert(keycode, pressed == ElementState::Pressed);
                    }
                }
            },

            WindowEvent::MouseInput { device_id: _, state, button } => {
                match (button, state) {
                    (MouseButton::Right, ElementState::Pressed) => {
                        let ray = self.player.forward_ray();
                        let looking_at = match self.player.get_looking_at(&self.gameobjects) {
                            Some(t) => t,
                            None => return
                        };
    
                        let new_center = looking_at.gameobject.get_pos() - looking_at.renderable.normal(&ray.at(looking_at.t));
                        
                        self.gameobjects.insert(new_center, Box::new(gameobject::Block::new_dirt_block(&new_center)));
                    },
                    (MouseButton::Left, ElementState::Pressed) => {
                        let looking_at = match self.player.get_looking_at(&self.gameobjects) {
                            Some(t) => t,
                            None => return
                        };
                        
                        self.gameobjects.remove(&looking_at.gameobject.get_pos());
                    },
                    _ => {}
                }
            }

            _ => {}
        }
    }
}

fn main() {
    debug_assert_eq!(std::mem::size_of::<object::camera::GpuUniform>() % 256, 0);

    let args = Arguments::parse();
    let config: Config = args.into();

    let player = gameobject::Player::new(
        ds::Vector3::new(0.0, 4.0, 0.0),
        ds::Vector3::zero(),
        config.fov,
        (config.width as f64, config.height as f64),
    );

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(config, player);

    event_loop.run_app(&mut app).expect("Event loop failed");
}