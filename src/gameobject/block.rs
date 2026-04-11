use render_engine::object::{Quad, Renderable};
use render_engine::ds::Vector3;
use render_engine::material::GpuMaterial;
use crate::gameobject::GameObject;

pub struct Block {
    quads: Vec<Box<dyn Renderable>>,
}

impl Block {
    pub fn new_grass_block(pos: &Vector3) -> Self {
        Self {
            quads: vec![
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(1, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x+0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 0.0, 0.0, 1.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(1, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z+0.5), &Vector3::new( 0.0, 0.0,-1.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(1, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x+0.5, pos.y-0.5, pos.z+0.5), &Vector3::new(-1.0, 0.0, 0.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(1, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y+0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 0.0, 1.0), GpuMaterial::texture(2, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 0.0, 1.0), GpuMaterial::texture(0, 0, 0))),
            ]
        }
    }

    pub fn new_dirt_block(pos: &Vector3) -> Self {
        Self {
            quads: vec![
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x+0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 0.0, 0.0, 1.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z+0.5), &Vector3::new( 0.0, 0.0,-1.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x+0.5, pos.y-0.5, pos.z+0.5), &Vector3::new(-1.0, 0.0, 0.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y+0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 0.0, 1.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 0.0, 1.0), GpuMaterial::texture(0, 0, 0))),
            ]
        }
    }
}

impl GameObject for Block {
    fn get_renderables(&self) -> &Vec<Box<dyn render_engine::object::Renderable>> {
        return &self.quads;
    }
}