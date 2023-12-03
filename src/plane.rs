use gl_matrix4rust::vec3::{AsVec3, Vec3};

use crate::utils::{distance_point_and_plane, distance_point_and_plane_abs};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pop: Vec3,
    normal: Vec3,
}

impl Plane {
    pub fn new(pop: Vec3, normal: Vec3) -> Self {
        Self {
            pop,
            normal: normal.normalize(),
        }
    }

    pub fn point_on_plane(&self) -> &Vec3 {
        &self.pop
    }

    pub fn normal(&self) -> &Vec3 {
        &self.normal
    }

    pub fn distance_to_point(&self, p: &Vec3) -> f64 {
        distance_point_and_plane(p, &self.pop, &self.normal)
    }

    pub fn distance_to_point_abs(&self, p: &Vec3) -> f64 {
        distance_point_and_plane_abs(p, &self.pop, &self.normal)
    }
}
