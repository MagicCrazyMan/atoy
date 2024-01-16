use gl_matrix4rust::vec3::Vec3;

use crate::utils::{distance_point_and_plane, distance_point_and_plane_abs};

/// A 3-Dimensions plane defines by a point on plane and a normal.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pop: Vec3,
    normal: Vec3,
}

impl Plane {
    /// Constructs a new plane by a point on plane and a normal.
    pub fn new(pop: Vec3, normal: Vec3) -> Self {
        Self {
            pop,
            normal: normal.normalize(),
        }
    }

    /// Returns point on plane.
    pub fn point_on_plane(&self) -> &Vec3 {
        &self.pop
    }

    /// Returns normal.
    pub fn normal(&self) -> &Vec3 {
        &self.normal
    }

    /// Calculates distance between a point and this plane.
    /// Checks [`distance_point_and_plane`] for more details.
    pub fn distance_to_point(&self, p: &Vec3) -> f64 {
        distance_point_and_plane(p, &self.pop, &self.normal)
    }

    /// Calculates absolute distance between a point and this plane.
    /// Checks [`distance_point_and_plane_abs`] for more details.
    pub fn distance_to_point_abs(&self, p: &Vec3) -> f64 {
        distance_point_and_plane_abs(p, &self.pop, &self.normal)
    }
}
