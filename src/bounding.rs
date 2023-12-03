use std::cell::RefCell;

use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::{AsVec3, Vec3},
};

use crate::{
    frustum::{ViewingFrustum, ViewingFrustumPlaneKind},
    utils::{distance_point_and_plane, distance_point_and_plane_abs},
};

#[derive(Debug)]
pub struct BoundingVolume {
    previous_outside_plane: RefCell<Option<ViewingFrustumPlaneKind>>,
    kind: BoundingVolumeKind,
}

impl BoundingVolume {
    pub fn new(kind: BoundingVolumeKind) -> Self {
        Self {
            previous_outside_plane: RefCell::new(None),
            kind,
        }
    }

    pub fn kind(&self) -> BoundingVolumeKind {
        self.kind
    }

    pub fn cull(&self, frustum: &ViewingFrustum) -> Culling {
        let mut previous_outside_plane = self.previous_outside_plane.borrow_mut();

        let culling = match &self.kind {
            BoundingVolumeKind::BoundingSphere { center, radius } => {
                cull_sphere(frustum, previous_outside_plane.clone(), center, *radius)
            }
            BoundingVolumeKind::AxisAlignedBoundingBox {
                min_x,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
            } => cull_aabb(
                frustum,
                previous_outside_plane.clone(),
                *min_x,
                *max_x,
                *min_y,
                *max_y,
                *min_z,
                *max_z,
            ),
        };

        if let Culling::Outside(plane) = &culling {
            *previous_outside_plane = Some(*plane);
        }

        culling
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundingVolumeKind {
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

impl BoundingVolumeKind {
    // pub fn cull(&self, frustum: &ViewingFrustum) -> Culling {
    //     match self {
    //         BoundingVolumeKind::BoundingSphere { center, radius } => {
    //             cull_sphere(frustum, center, *radius)
    //         }
    //         BoundingVolumeKind::AxisAlignedBoundingBox {
    //             min_x,
    //             max_x,
    //             min_y,
    //             max_y,
    //             min_z,
    //             max_z,
    //         } => cull_aabb(frustum, *min_x, *max_x, *min_y, *max_y, *min_z, *max_z),
    //     }
    // }

    pub fn transform(&self, transformation: &Mat4) -> Self {
        match self {
            BoundingVolumeKind::BoundingSphere { center, radius } => {
                let scaling = transformation.scaling();
                let max = scaling.x().max(scaling.y()).max(scaling.z());
                BoundingVolumeKind::BoundingSphere {
                    center: center.transform_mat4(transformation),
                    radius: max * *radius,
                }
            }
            BoundingVolumeKind::AxisAlignedBoundingBox {
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

                BoundingVolumeKind::AxisAlignedBoundingBox {
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

/// Culls bounding sphere from frustum by calculating distance between sphere center to each plane of frustum.
fn cull_sphere(
    frustum: &ViewingFrustum,
    previous_outside_plane: Option<ViewingFrustumPlaneKind>,
    center: &Vec3,
    radius: f64,
) -> Culling {
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

    let mut planes = [
        (ViewingFrustumPlaneKind::Top, Some(frustum.top())),
        (ViewingFrustumPlaneKind::Bottom, Some(frustum.bottom())),
        (ViewingFrustumPlaneKind::Left, Some(frustum.left())),
        (ViewingFrustumPlaneKind::Right, Some(frustum.right())),
        (ViewingFrustumPlaneKind::Near, Some(frustum.near())),
        // far plane may not exists
        (ViewingFrustumPlaneKind::Far, frustum.far()),
    ];
    // puts previous outside plane to the top if exists
    if let Some(previous) = previous_outside_plane {
        planes.swap(0, previous as usize);
    }
    for (kind, plane) in planes.iter() {
        match plane {
            Some(plane) => {
                let distance = plane.distance_to_point(center);
                if distance > radius {
                    // outside, return
                    return Culling::Outside(*kind);
                } else if distance < -radius {
                    // inside
                    inside_count += 1;
                    distances[*kind as usize] = Some(distance + radius);
                } else {
                    // intersect, do nothing
                    distances[*kind as usize] = Some(distance);
                }
            }
            None => {
                // only far plane reaches here, regards as inside
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

/// [Optimized View Frustum Culling Algorithms for Bounding Boxes](https://www.cse.chalmers.se/~uffe/vfc_bbox.pdf)
fn cull_aabb(
    frustum: &ViewingFrustum,
    previous_outside_plane: Option<ViewingFrustumPlaneKind>,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    min_z: f64,
    max_z: f64,
) -> Culling {
    let center = Vec3::from_values(
        (min_x + max_x) / 2.0,
        (min_y + max_y) / 2.0,
        (min_z + max_z) / 2.0,
    );
    let mut planes = [
        (ViewingFrustumPlaneKind::Top, Some(frustum.top())),
        (ViewingFrustumPlaneKind::Bottom, Some(frustum.bottom())),
        (ViewingFrustumPlaneKind::Left, Some(frustum.left())),
        (ViewingFrustumPlaneKind::Right, Some(frustum.right())),
        (ViewingFrustumPlaneKind::Near, Some(frustum.near())),
        // far plane may not exists
        (ViewingFrustumPlaneKind::Far, frustum.far()),
    ];
    // puts previous outside plane to the top if exists
    if let Some(previous) = previous_outside_plane {
        planes.swap(0, previous as usize);
    }

    let mut distances = [
        Some(0.0),
        Some(0.0),
        Some(0.0),
        Some(0.0),
        Some(0.0),
        Some(0.0),
        None,
    ];
    let mut intersect = false;
    for (kind, plane) in planes.iter() {
        match plane {
            Some(plane) => unsafe {
                let point_on_plane = plane.point_on_plane();
                let n = plane.normal();
                let nx = n.x();
                let ny = n.y();
                let nz = n.z();

                // finds n- and p-vertices
                let mut signs = 0u8;
                signs |= (std::mem::transmute::<f64, u64>(nx) >> 63) as u8 & 0b00000001;
                signs |= (std::mem::transmute::<f64, u64>(ny) >> 62) as u8 & 0b00000010;
                signs |= (std::mem::transmute::<f64, u64>(nz) >> 61) as u8 & 0b00000100;

                let (nv, pv) = match signs {
                    0b000 => (
                        Vec3::from_values(min_x, min_y, min_z),
                        Vec3::from_values(max_x, max_y, max_z),
                    ),
                    0b001 => (
                        Vec3::from_values(max_x, min_y, min_z),
                        Vec3::from_values(min_x, max_y, max_z),
                    ),
                    0b010 => (
                        Vec3::from_values(min_x, max_y, min_z),
                        Vec3::from_values(max_x, min_y, max_z),
                    ),
                    0b011 => (
                        Vec3::from_values(max_x, max_y, min_z),
                        Vec3::from_values(min_x, min_y, max_z),
                    ),
                    0b100 => (
                        Vec3::from_values(min_x, min_y, max_z),
                        Vec3::from_values(max_x, max_y, min_z),
                    ),
                    0b101 => (
                        Vec3::from_values(max_x, min_y, max_z),
                        Vec3::from_values(min_x, max_y, min_z),
                    ),
                    0b110 => (
                        Vec3::from_values(min_x, max_y, max_z),
                        Vec3::from_values(max_x, min_y, min_z),
                    ),
                    0b111 => (
                        Vec3::from_values(max_x, max_y, max_z),
                        Vec3::from_values(min_x, min_y, min_z),
                    ),
                    _ => unreachable!(),
                };

                let d = distance_point_and_plane_abs(&nv, &center, n);
                let a = distance_point_and_plane(&nv, &point_on_plane, n) - d;
                if a > 0.0 {
                    return Culling::Outside(*kind);
                }
                let b = distance_point_and_plane(&pv, &point_on_plane, n) + d;
                if b > 0.0 {
                    intersect = true;
                }

                // uses distance between center of bounding volume and plane
                distances[*kind as usize] =
                    Some(distance_point_and_plane(&center, &point_on_plane, n));
            },
            None => {
                // only far plane reaches here, regards as inside, do nothing.
            }
        }
    }

    if intersect {
        Culling::Intersect {
            top: distances[0].unwrap(),
            bottom: distances[1].unwrap(),
            left: distances[2].unwrap(),
            right: distances[3].unwrap(),
            near: distances[4].unwrap(),
            far: distances[5],
        }
    } else {
        Culling::Inside {
            top: distances[0].unwrap(),
            bottom: distances[1].unwrap(),
            left: distances[2].unwrap(),
            right: distances[3].unwrap(),
            near: distances[4].unwrap(),
            far: distances[5],
        }
    }
}

#[test]
fn bitfields() {
    let nx = -1.0;
    let ny = -2.0;
    let nz = -3.0;

    let mut signs = 0u8;
    unsafe {
        signs |= (std::mem::transmute::<f64, u64>(nx) >> 63) as u8 & 0b00000001;
        signs |= (std::mem::transmute::<f64, u64>(ny) >> 62) as u8 & 0b00000010;
        signs |= (std::mem::transmute::<f64, u64>(nz) >> 61) as u8 & 0b00000100;
    }

    println!("{:08b}", signs);
}

/// Culling status of a [`BoundingVolume`] and a [`ViewingFrustum`]
/// with shortest distance of each plane if inside or intersect.
///
/// Far value is none If viewing frustum accepts entities infinity far away
/// ([`Camera::far()`](crate::camera::Camera::far) is none).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Culling {
    Outside(ViewingFrustumPlaneKind),
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
