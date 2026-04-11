use render_engine::object::Renderable;

pub mod block;

pub use block::Block;

pub trait GameObject {
    fn get_renderables(&self) -> &Vec<Box<dyn Renderable>>;
}

pub enum Faces {
    Front,
    Back,
    Left,
    Right,
    Up,
    Down
}