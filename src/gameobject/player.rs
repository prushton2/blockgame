use std::collections::HashMap;

use render_engine::{ds::{self, Vector3}, object::{self, Renderable}};

use crate::gameobject::GameObject;

pub struct Player {
    camera: object::Camera,
    rotation: ds::Vector3,
}

impl Player {
    pub fn new(pos: ds::Vector3, rotation: ds::Vector3, vfov: f64, window_dimensions: (f64, f64)) -> Self {
        let mut this = Self {
            camera: object::Camera::new(pos, 3.0, window_dimensions, vfov),
            rotation: ds::Vector3::zero()
        };

        this.change_rotation(rotation);
        this.camera.update_outputs();
        this
    }

    pub fn change_rotation(&mut self, delta: ds::Vector3) {
        self.rotation = self.rotation + delta;

        let pitch_rotation = ds::Vector3::new(
            0.0,
            self.rotation.pitch().sin(),
            self.rotation.pitch().cos()
        );

        let full_rotation = ds::Vector3::new(
            self.rotation.yaw().sin() * pitch_rotation.z,
            pitch_rotation.y,
            self.rotation.yaw().cos() * pitch_rotation.z
        );

        self.camera.set_dir_relative(full_rotation);
    }
    
    pub fn move_player(&mut self, delta: &ds::Vector3) {

        let sin = self.rotation.yaw().sin();
        let cos = self.rotation.yaw().cos();

        self.camera.move_camera(ds::Vector3::new(
            delta.z * sin + delta.x * cos,
            delta.y,
            delta.z * cos - delta.x * sin
        ));
    }

    pub fn forward_ray(&self) -> ds::Ray {
        ds::Ray::new(&self.get_camera().pos(), &(self.get_camera().dir() - self.get_camera().pos()))
    }

    pub fn get_looking_at<'a>(&self, world: &'a HashMap<Vector3, Box<dyn GameObject>>) -> Option<LookingAt<'a>> {
        let ray = self.forward_ray();
        let mut closest: Option<LookingAt> = None;

        for (_pos, go) in world {
            let intersection = go.intersects(&ray);

            if intersection.is_none() {
                continue;
            }

            if closest.is_none() || intersection.unwrap().0 < closest.unwrap().t{
                closest = Some(LookingAt {
                    t: intersection.unwrap().0,
                    gameobject: go.as_ref(),
                    renderable: intersection.unwrap().1
                });
            }
        }

        return closest;
    }

    // pub fn get_looking_at(&self, )

    pub fn get_rotation(&self) -> ds::Vector3 {
        self.rotation
    }

    pub fn get_camera(&self) -> &object::Camera {
        &self.camera
    }

    pub fn get_camera_mut(&mut self) -> &mut object::Camera {
        &mut self.camera
    }

    pub fn update_outputs(&mut self) {
        self.camera.update_outputs();
    }
}

#[derive(Clone, Copy)]
pub struct LookingAt<'a> {
    pub t:          f64,
    pub gameobject: &'a dyn GameObject,
    pub renderable: &'a dyn Renderable
}