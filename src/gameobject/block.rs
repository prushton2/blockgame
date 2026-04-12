use render_engine::object::{Quad, Renderable};
use render_engine::ds::{Aabb, Ray, Vector3};
use render_engine::material::GpuMaterial;
use crate::gameobject::GameObject;

pub struct Block {
    quads: Vec<Box<dyn Renderable>>,
    pos: Vector3,
}

impl Block {
    pub fn new_grass_block(pos: &Vector3) -> Self {
        Self {
            pos: pos.clone(),
            quads: vec![
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(1, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x+0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 0.0, 0.0, 1.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(1, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z+0.5), &Vector3::new( 0.0, 0.0,-1.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(1, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x+0.5, pos.y-0.5, pos.z+0.5), &Vector3::new(-1.0, 0.0, 0.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(1, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y+0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 0.0, 1.0), GpuMaterial::texture(2, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 0.0, 0.0, 1.0), &Vector3::new(1.0, 0.0, 0.0), GpuMaterial::texture(0, 0, 0))),
            ]
        }
    }

    pub fn new_dirt_block(pos: &Vector3) -> Self {
        Self {
            pos: pos.clone(),
            quads: vec![
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x+0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 0.0, 0.0, 1.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z+0.5), &Vector3::new( 0.0, 0.0,-1.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x+0.5, pos.y-0.5, pos.z+0.5), &Vector3::new(-1.0, 0.0, 0.0), &Vector3::new(0.0, 1.0, 0.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y+0.5, pos.z-0.5), &Vector3::new( 1.0, 0.0, 0.0), &Vector3::new(0.0, 0.0, 1.0), GpuMaterial::texture(0, 0, 0))),
                Box::new(Quad::new(&Vector3::new(pos.x-0.5, pos.y-0.5, pos.z-0.5), &Vector3::new( 0.0, 0.0, 1.0), &Vector3::new(1.0, 0.0, 0.0), GpuMaterial::texture(0, 0, 0))),
            ]
        }
    }
}

impl GameObject for Block {
    fn get_renderables(&self) -> &Vec<Box<dyn render_engine::object::Renderable>> {
        return &self.quads;
    }

    fn intersects(&self, ray: &Ray) -> Option<(f64, &dyn Renderable)> {
        if Aabb::from_vector3(
            &(self.pos + Vector3::new(-0.5, -0.5, -0.5)),
            &(self.pos + Vector3::new(0.5, 0.5, 0.5))
        ).intersects(ray).is_none() {
            return None;
        }
        let mut lowest_t: Option<f64> = None;
        let mut reference: Option<&dyn Renderable> = None;

        for quad in &self.quads {
            match quad.intersects(ray) {
                None => continue,
                Some(t) => {
                    if lowest_t.is_none() || t < lowest_t.unwrap() {
                        reference = Some(quad.as_ref());
                        lowest_t = Some(t);
                    }
                }
            }
        }

        match reference {
            Some(t) => return Some((lowest_t.unwrap(), t)),
            None => return None
        }
    }
}