use gl_matrix4rust::{
    mat4::Mat4,
    vec3::{AsVec3, Vec3},
};

use crate::frustum::ViewingFrustum;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundingVolume {
    Sphere { origin: Vec3, radius: f64 },
}

impl BoundingVolume {
    pub fn cull(&self, frustum: &ViewingFrustum) -> Culling {
        match self {
            BoundingVolume::Sphere { origin, radius } => cull_sphere(frustum, origin, *radius),
        }
    }

    pub fn transform(&self, transformation: &Mat4) -> Self {
        match self {
            BoundingVolume::Sphere { origin, radius } => BoundingVolume::Sphere {
                origin: origin.transform_mat4(transformation),
                radius: *radius,
            },
        }
    }
}

fn cull_sphere(frustum: &ViewingFrustum, origin: &Vec3, radius: f64) -> Culling {
    let mut inside_count = 0u8;
    let mut distances = [0.0; 6];
    let planes = [
        frustum.top(),
        frustum.bottom(),
        frustum.left(),
        frustum.right(),
        frustum.near(),
    ];
    for (index, plane) in planes.iter().enumerate() {
        let distance = plane.distance_to_point(origin);
        if distance > radius {
            // outside, return
            return Culling::Outside;
        } else if distance < -radius {
            // inside
            inside_count += 1;
            distances[index] = distance + radius;
        } else {
            // intersect, do nothing
            distances[index] = distance;
        }
    }

    // separates far plane
    let far_distance = match frustum.far() {
        Some(far) => {
            let distance = far.distance_to_point(origin);
            if distance > radius {
                // outside, return
                return Culling::Outside;
            } else if distance < -radius {
                // inside
                inside_count += 1;
                Some(distance + radius)
            } else {
                // intersect, do nothing
                Some(distance)
            }
        }
        None => {
            // if no far value (infinity far away), regards as inside
            inside_count += 1;
            None
        }
    };

    if inside_count == 6 {
        Culling::Inside {
            top: distances[0],
            bottom: distances[1],
            left: distances[2],
            right: distances[3],
            near: distances[4],
            far: far_distance,
        }
    } else {
        Culling::Intersect {
            top: distances[0],
            bottom: distances[1],
            left: distances[2],
            right: distances[3],
            near: distances[4],
            far: far_distance,
        }
    }
}

/// Culling status of a [`BoundingVolume`] and a [`ViewingFrustum`]
/// with shortest distance of each plane if inside or intersect.
///
/// Far value is none If viewing frustum accepts entities infinity far away
/// ([`Camera::far()`](crate::camera::Camera::far) is none).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Culling {
    Outside,
    Inside {
        near: f64,
        far: Option<f64>,
        top: f64,
        bottom: f64,
        left: f64,
        right: f64,
    },
    Intersect {
        near: f64,
        far: Option<f64>,
        top: f64,
        bottom: f64,
        left: f64,
        right: f64,
    },
}
