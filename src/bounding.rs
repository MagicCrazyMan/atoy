use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::{AsVec3, Vec3},
};

use crate::frustum::ViewingFrustum;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundingVolume {
    BoundingSphere {
        center: Vec3,
        radius: f64,
    },
    AxisAlignedBoundingBox {
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
        min_z: f64,
        max_z: f64,
    },
}

impl BoundingVolume {
    pub fn cull(&self, frustum: &ViewingFrustum) -> Culling {
        match self {
            BoundingVolume::BoundingSphere { center, radius } => {
                cull_sphere(frustum, center, *radius)
            }
            BoundingVolume::AxisAlignedBoundingBox {
                min_x,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
            } => cull_aabb(frustum, *min_x, *max_x, *min_y, *max_y, *min_z, *max_z),
        }
    }

    pub fn transform(&self, transformation: &Mat4) -> Self {
        match self {
            BoundingVolume::BoundingSphere { center, radius } => {
                let scaling = transformation.scaling();
                let max = scaling.x().max(scaling.y()).max(scaling.z());
                BoundingVolume::BoundingSphere {
                    center: center.transform_mat4(transformation),
                    radius: max * *radius,
                }
            }
            BoundingVolume::AxisAlignedBoundingBox {
                min_x,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
            } => {
                // constructs 8 vertices and applies transform to them
                // and then finds AABB from the transformed 8 vertices
                let min_x = *min_x;
                let max_x = *max_x;
                let min_y = *min_y;
                let max_y = *max_y;
                let min_z = *min_z;
                let max_z = *max_z;

                let p0 = (min_x, min_y, min_z).transform_mat4(transformation);
                let p1 = (min_x, max_y, min_z).transform_mat4(transformation);
                let p2 = (min_x, min_y, max_z).transform_mat4(transformation);
                let p3 = (min_x, max_y, max_z).transform_mat4(transformation);
                let p4 = (max_x, max_y, min_z).transform_mat4(transformation);
                let p5 = (max_x, min_y, max_z).transform_mat4(transformation);
                let p6 = (max_x, min_y, min_z).transform_mat4(transformation);
                let p7 = (max_x, max_y, max_z).transform_mat4(transformation);

                let x = [p0.0, p1.0, p2.0, p3.0, p4.0, p5.0, p6.0, p7.0];
                let y = [p0.1, p1.1, p2.1, p3.1, p4.1, p5.1, p6.1, p7.1];
                let z = [p0.2, p1.2, p2.2, p3.2, p4.2, p5.2, p6.2, p7.2];
                let min_x = *x.iter().min_by(|a, b| a.total_cmp(&b)).unwrap();
                let max_x = *x.iter().max_by(|a, b| a.total_cmp(&b)).unwrap();
                let min_y = *y.iter().min_by(|a, b| a.total_cmp(&b)).unwrap();
                let max_y = *y.iter().max_by(|a, b| a.total_cmp(&b)).unwrap();
                let min_z = *z.iter().min_by(|a, b| a.total_cmp(&b)).unwrap();
                let max_z = *z.iter().max_by(|a, b| a.total_cmp(&b)).unwrap();

                BoundingVolume::AxisAlignedBoundingBox {
                    min_x,
                    max_x,
                    min_y,
                    max_y,
                    min_z,
                    max_z,
                }
            }
        }
    }
}

fn cull_sphere(frustum: &ViewingFrustum, center: &Vec3, radius: f64) -> Culling {
    let mut inside_count = 0u8;
    let mut distances = [
        Some(0.0),
        Some(0.0),
        Some(0.0),
        Some(0.0),
        Some(0.0),
        Some(0.0),
        None,
    ];
    let planes = [
        Some(frustum.top()),
        Some(frustum.bottom()),
        Some(frustum.left()),
        Some(frustum.right()),
        Some(frustum.near()),
        frustum.far(), // far plane may not exists
    ];
    for (index, plane) in planes.iter().enumerate() {
        match plane {
            Some(plane) => {
                let distance = plane.distance_to_point(center);
                if distance > radius {
                    // outside, return
                    return Culling::Outside;
                } else if distance < -radius {
                    // inside
                    inside_count += 1;
                    distances[index] = Some(distance + radius);
                } else {
                    // intersect, do nothing
                    distances[index] = Some(distance);
                }
            }
            None => {
                // if no plane (far plane), regards as inside
                inside_count += 1;
            }
        }
    }

    if inside_count == 6 {
        Culling::Inside {
            top: distances[0].unwrap(),
            bottom: distances[1].unwrap(),
            left: distances[2].unwrap(),
            right: distances[3].unwrap(),
            near: distances[4].unwrap(),
            far: distances[5],
        }
    } else {
        Culling::Intersect {
            top: distances[0].unwrap(),
            bottom: distances[1].unwrap(),
            left: distances[2].unwrap(),
            right: distances[3].unwrap(),
            near: distances[4].unwrap(),
            far: distances[5],
        }
    }
}

fn cull_aabb(
    frustum: &ViewingFrustum,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    min_z: f64,
    max_z: f64,
) -> Culling {
    let mut signs = 0;
    todo!()
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
