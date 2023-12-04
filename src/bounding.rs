use std::cell::RefCell;

use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::{AsVec3, Vec3},
};

use crate::{
    frustum::ViewingFrustum,
    utils::{distance_point_and_plane, distance_point_and_plane_abs},
};

#[derive(Debug)]
pub struct BoundingVolume {
    previous_outside_plane: RefCell<Option<ViewingFrustumPlane>>,
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
            BoundingVolumeKind::OrientedBoundingBox(matrix) => {
                cull_obb(frustum, previous_outside_plane.clone(), matrix)
            }
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
    /// An OBB is defined as a model matrix only.
    /// When we need to restore vertex of the OBB,
    /// we will apply the model matrix to to standard AABB
    /// (with vertices `(1, 1, 1)`, `(1, 1, -1)`, `(-1, 1, 1)` and etc.).
    ///
    /// But storing a center as Vec3, a rotation and scaling together as Mat3 maybe a better idea.
    /// Since this saves 4 bytes than Mat4
    OrientedBoundingBox(Mat4),
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
            BoundingVolumeKind::OrientedBoundingBox(matrix) => {
                BoundingVolumeKind::OrientedBoundingBox(*transformation * *matrix)
            }
        }
    }
}

/// Culls bounding sphere from frustum by calculating distance between sphere center to each plane of frustum.
fn cull_sphere(
    frustum: &ViewingFrustum,
    previous_outside_plane: Option<ViewingFrustumPlane>,
    center: &Vec3,
    radius: f64,
) -> Culling {
    let mut inside_count = 0u8;
    let mut distances = [None, None, None, None, None, None, None];

    let mut planes = [
        (ViewingFrustumPlane::Top, Some(frustum.top())),
        (ViewingFrustumPlane::Bottom, Some(frustum.bottom())),
        (ViewingFrustumPlane::Left, Some(frustum.left())),
        (ViewingFrustumPlane::Right, Some(frustum.right())),
        (ViewingFrustumPlane::Near, Some(frustum.near())),
        // far plane may not exists
        (ViewingFrustumPlane::Far, frustum.far()),
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
            top: distances[ViewingFrustumPlane::Top as usize].unwrap(),
            bottom: distances[ViewingFrustumPlane::Bottom as usize].unwrap(),
            left: distances[ViewingFrustumPlane::Left as usize].unwrap(),
            right: distances[ViewingFrustumPlane::Right as usize].unwrap(),
            near: distances[ViewingFrustumPlane::Near as usize].unwrap(),
            far: distances[ViewingFrustumPlane::Far as usize],
        }
    } else {
        Culling::Intersect {
            top: distances[ViewingFrustumPlane::Top as usize].unwrap(),
            bottom: distances[ViewingFrustumPlane::Bottom as usize].unwrap(),
            left: distances[ViewingFrustumPlane::Left as usize].unwrap(),
            right: distances[ViewingFrustumPlane::Right as usize].unwrap(),
            near: distances[ViewingFrustumPlane::Near as usize].unwrap(),
            far: distances[ViewingFrustumPlane::Far as usize],
        }
    }
}

/// [Optimized View Frustum Culling Algorithms for Bounding Boxes](https://www.cse.chalmers.se/~uffe/vfc_bbox.pdf)
fn cull_aabb(
    frustum: &ViewingFrustum,
    previous_outside_plane: Option<ViewingFrustumPlane>,
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
        (ViewingFrustumPlane::Top, Some(frustum.top())),
        (ViewingFrustumPlane::Bottom, Some(frustum.bottom())),
        (ViewingFrustumPlane::Left, Some(frustum.left())),
        (ViewingFrustumPlane::Right, Some(frustum.right())),
        (ViewingFrustumPlane::Near, Some(frustum.near())),
        // far plane may not exists
        (ViewingFrustumPlane::Far, frustum.far()),
    ];
    // puts previous outside plane to the top if exists
    if let Some(previous) = previous_outside_plane {
        planes.swap(0, previous as usize);
    }

    let vertices = [
        Vec3::from_values(max_x, max_y, max_z), // 000
        Vec3::from_values(min_x, max_y, max_z), // 001
        Vec3::from_values(max_x, min_y, max_z), // 010
        Vec3::from_values(min_x, min_y, max_z), // 011
        Vec3::from_values(max_x, max_y, min_z), // 100
        Vec3::from_values(min_x, max_y, min_z), // 101
        Vec3::from_values(max_x, min_y, min_z), // 110
        Vec3::from_values(min_x, min_y, min_z), // 111
    ];
    let mut distances = [None, None, None, None, None, None];
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
                let pi = signs;
                let ni = !signs & 0b00000111;
                let pv = &vertices[pi as usize];
                let nv = &vertices[ni as usize];

                let d = distance_point_and_plane_abs(nv, &center, n);
                let a = distance_point_and_plane(nv, &point_on_plane, n) - d;
                if a > 0.0 {
                    return Culling::Outside(*kind);
                }
                let b = distance_point_and_plane(pv, &point_on_plane, n) + d;
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
            top: distances[ViewingFrustumPlane::Top as usize].unwrap(),
            bottom: distances[ViewingFrustumPlane::Bottom as usize].unwrap(),
            left: distances[ViewingFrustumPlane::Left as usize].unwrap(),
            right: distances[ViewingFrustumPlane::Right as usize].unwrap(),
            near: distances[ViewingFrustumPlane::Near as usize].unwrap(),
            far: distances[ViewingFrustumPlane::Far as usize],
        }
    } else {
        Culling::Inside {
            top: distances[ViewingFrustumPlane::Top as usize].unwrap(),
            bottom: distances[ViewingFrustumPlane::Bottom as usize].unwrap(),
            left: distances[ViewingFrustumPlane::Left as usize].unwrap(),
            right: distances[ViewingFrustumPlane::Right as usize].unwrap(),
            near: distances[ViewingFrustumPlane::Near as usize].unwrap(),
            far: distances[ViewingFrustumPlane::Far as usize],
        }
    }
}

/// [Optimized View Frustum Culling Algorithms for Bounding Boxes](https://www.cse.chalmers.se/~uffe/vfc_bbox.pdf)
fn cull_obb(
    frustum: &ViewingFrustum,
    previous_outside_plane: Option<ViewingFrustumPlane>,
    matrix: &Mat4,
) -> Culling {
    let mut planes = [
        (ViewingFrustumPlane::Top, Some(frustum.top())),
        (ViewingFrustumPlane::Bottom, Some(frustum.bottom())),
        (ViewingFrustumPlane::Left, Some(frustum.left())),
        (ViewingFrustumPlane::Right, Some(frustum.right())),
        (ViewingFrustumPlane::Near, Some(frustum.near())),
        // far plane may not exists
        (ViewingFrustumPlane::Far, frustum.far()),
    ];
    // puts previous outside plane to the top if exists
    if let Some(previous) = previous_outside_plane {
        planes.swap(0, previous as usize);
    }

    let center = matrix.translation();
    let mut vertices = [
        None, // 000
        None, // 001
        None, // 010
        None, // 011
        None, // 100
        None, // 101
        None, // 110
        None, // 111
    ]; // lazy evaluation
    let mut distances = [None, None, None, None, None, None];
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
                let pi = signs;
                let ni = !signs & 0b00000111;
                let pv = *vertices[pi as usize].get_or_insert_with(|| {
                    let x = if pi & 0b00000001 == 0 { 1.0 } else { -1.0 };
                    let y = if pi & 0b00000010 == 0 { 1.0 } else { -1.0 };
                    let z = if pi & 0b00000100 == 0 { 1.0 } else { -1.0 };
                    Vec3::from_values(x, y, z).transform_mat4(matrix)
                });
                let nv = vertices[ni as usize].get_or_insert_with(|| {
                    let x = if ni & 0b00000001 == 0 { 1.0 } else { -1.0 };
                    let y = if ni & 0b00000010 == 0 { 1.0 } else { -1.0 };
                    let z = if ni & 0b00000100 == 0 { 1.0 } else { -1.0 };
                    Vec3::from_values(x, y, z).transform_mat4(matrix)
                });

                let d = distance_point_and_plane_abs(nv, &center, n);
                let a = distance_point_and_plane(nv, &point_on_plane, n) - d;
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
            top: distances[ViewingFrustumPlane::Top as usize].unwrap(),
            bottom: distances[ViewingFrustumPlane::Bottom as usize].unwrap(),
            left: distances[ViewingFrustumPlane::Left as usize].unwrap(),
            right: distances[ViewingFrustumPlane::Right as usize].unwrap(),
            near: distances[ViewingFrustumPlane::Near as usize].unwrap(),
            far: distances[ViewingFrustumPlane::Far as usize],
        }
    } else {
        Culling::Inside {
            top: distances[ViewingFrustumPlane::Top as usize].unwrap(),
            bottom: distances[ViewingFrustumPlane::Bottom as usize].unwrap(),
            left: distances[ViewingFrustumPlane::Left as usize].unwrap(),
            right: distances[ViewingFrustumPlane::Right as usize].unwrap(),
            near: distances[ViewingFrustumPlane::Near as usize].unwrap(),
            far: distances[ViewingFrustumPlane::Far as usize],
        }
    }
}

#[test]
fn bitfields() {
    let nx = -1.0;
    let ny = 2.0;
    let nz = -3.0;

    let mut signs = 0u8;
    unsafe {
        signs |= (std::mem::transmute::<f64, u64>(nx) >> 63) as u8 & 0b00000001;
        signs |= (std::mem::transmute::<f64, u64>(ny) >> 62) as u8 & 0b00000010;
        signs |= (std::mem::transmute::<f64, u64>(nz) >> 61) as u8 & 0b00000100;
    }

    println!("{:08b}", !signs);
}

/// Culling status of a [`BoundingVolume`] and a [`ViewingFrustum`]
/// with shortest distance of each plane if inside or intersect.
///
/// Far value is none If viewing frustum accepts entities infinity far away
/// ([`Camera::far()`](crate::camera::Camera::far) is none).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Culling {
    Outside(ViewingFrustumPlane),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ViewingFrustumPlane {
    Top = 0,
    Bottom = 1,
    Left = 2,
    Right = 3,
    Near = 4,
    Far = 5,
}
