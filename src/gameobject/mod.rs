use render_engine::{ds::Ray, object::Renderable};

pub mod block;
pub mod player;

pub use block::Block;
pub use player::Player;

pub trait GameObject {
    fn get_renderables(&self) -> &Vec<Box<dyn Renderable>>;
    fn intersects(&self, ray: &Ray) -> Option<(f64, &dyn Renderable)>;
}

pub enum Faces {
    Front,
    Back,
    Left,
    Right,
    Up,
    Down
}