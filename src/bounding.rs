use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::{AsVec3, Vec3},
};

use crate::{
    frustum::{FrustumPlaneIndex, ViewFrustum},
    plane::Plane,
    utils::{distance_point_and_plane, distance_point_and_plane_abs},
};

/// An bounding volume for culling detection purpose.
/// This structure collects more information than [`BoundingVolume`]
/// to speed up the culling detection procedure.
#[derive(Debug)]
pub struct CullingBoundingVolume {
    previous_outside_plane: Option<FrustumPlaneIndex>,
    bounding: BoundingVolume,
}

impl CullingBoundingVolume {
    /// Constructs a new bounding volume from a [`BoundingVolume`].
    pub fn new(bounding: BoundingVolume) -> Self {
        Self {
            previous_outside_plane: None,
            bounding,
        }
    }

    /// Gets the [`BoundingVolume`] associated with this bounding volume.
    pub fn bounding(&self) -> BoundingVolume {
        self.bounding
    }

    /// Applies culling detection against a frustum.
    pub fn cull(&mut self, frustum: &ViewFrustum) -> Culling {
        let mut planes: [(FrustumPlaneIndex, Option<&Plane>); 6] = [
            (FrustumPlaneIndex::Top, Some(frustum.top())),
            (FrustumPlaneIndex::Bottom, Some(frustum.bottom())),
            (FrustumPlaneIndex::Left, Some(frustum.left())),
            (FrustumPlaneIndex::Right, Some(frustum.right())),
            (FrustumPlaneIndex::Near, Some(frustum.near())),
            (FrustumPlaneIndex::Far, frustum.far()), // far plane may not exists
        ];
        if let Some(p) = self.previous_outside_plane {
            planes.swap(0, p as usize);
        }

        let culling = match &self.bounding {
            BoundingVolume::BoundingSphere { center, radius } => {
                cull_sphere(planes, center, *radius)
            }
            BoundingVolume::AxisAlignedBoundingBox {
                min_x,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
            } => cull_bb(
                planes,
                &Vec3::from_values(
                    (min_x + max_x) / 2.0,
                    (min_y + max_y) / 2.0,
                    (min_z + max_z) / 2.0,
                ),
                |signs| match signs {
                    0b000 => Vec3::from_values(*max_x, *max_y, *max_z), // 000
                    0b001 => Vec3::from_values(*min_x, *max_y, *max_z), // 001
                    0b010 => Vec3::from_values(*max_x, *min_y, *max_z), // 010
                    0b011 => Vec3::from_values(*min_x, *min_y, *max_z), // 011
                    0b100 => Vec3::from_values(*max_x, *max_y, *min_z), // 100
                    0b101 => Vec3::from_values(*min_x, *max_y, *min_z), // 101
                    0b110 => Vec3::from_values(*max_x, *min_y, *min_z), // 110
                    0b111 => Vec3::from_values(*min_x, *min_y, *min_z), // 111
                    _ => unreachable!(),
                },
            ),
            BoundingVolume::OrientedBoundingBox { center, x, y, z } => {
                let center = *center;
                let x = *x;
                let y = *y;
                let z = *z;
                cull_bb(planes, &center, |signs| match signs {
                    0b000 => center + x + y + z, // 000
                    0b001 => center + x + y - z, // 001
                    0b010 => center + x - y + z, // 010
                    0b011 => center + x - y - z, // 011
                    0b100 => center - x + y + z, // 100
                    0b101 => center - x + y - z, // 101
                    0b110 => center - x - y + z, // 110
                    0b111 => center - x - y - z, // 111
                    _ => unreachable!(),
                })
            }
        };

        if let Culling::Outside(plane) = &culling {
            self.previous_outside_plane = Some(*plane);
        }

        culling
    }

    /// Gets center of this bounding volume.
    pub fn center(&self) -> Vec3 {
        self.bounding.center()
    }

    /// Transforms this bounding volume native by a transformation matrix.
    pub fn transform(&mut self, transformation: &Mat4) {
        self.bounding = self.bounding.transform(transformation);
        self.previous_outside_plane = None;
    }
}

/// Available bounding volumes.
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
    /// XYZ axes are orthogonal half axes of the oriented bounding box.
    OrientedBoundingBox {
        center: Vec3,
        x: Vec3,
        y: Vec3,
        z: Vec3,
    },
}

impl BoundingVolume {
    /// Gets center of this bounding volume.
    pub fn center(&self) -> Vec3 {
        match self {
            BoundingVolume::BoundingSphere { center, .. } => *center,
            BoundingVolume::AxisAlignedBoundingBox {
                min_x,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
            } => Vec3::from_values(
                (min_x + max_x) / 2.0,
                (min_y + max_y) / 2.0,
                (min_z + max_z) / 2.0,
            ),
            BoundingVolume::OrientedBoundingBox { center, .. } => *center,
        }
    }

    /// Applies culling detection against a frustum.
    pub fn cull(&self, frustum: &ViewFrustum) -> Culling {
        let mut planes: [(FrustumPlaneIndex, Option<&Plane>); 6] = [
            (FrustumPlaneIndex::Top, Some(frustum.top())),
            (FrustumPlaneIndex::Bottom, Some(frustum.bottom())),
            (FrustumPlaneIndex::Left, Some(frustum.left())),
            (FrustumPlaneIndex::Right, Some(frustum.right())),
            (FrustumPlaneIndex::Near, Some(frustum.near())),
            (FrustumPlaneIndex::Far, frustum.far()),
        ];

        match self {
            BoundingVolume::BoundingSphere { center, radius } => {
                cull_sphere(planes, center, *radius)
            }
            BoundingVolume::AxisAlignedBoundingBox {
                min_x,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
            } => cull_bb(
                planes,
                &Vec3::from_values(
                    (min_x + max_x) / 2.0,
                    (min_y + max_y) / 2.0,
                    (min_z + max_z) / 2.0,
                ),
                |signs| match signs {
                    0b000 => Vec3::from_values(*max_x, *max_y, *max_z),
                    0b001 => Vec3::from_values(*min_x, *max_y, *max_z),
                    0b010 => Vec3::from_values(*max_x, *min_y, *max_z),
                    0b011 => Vec3::from_values(*min_x, *min_y, *max_z),
                    0b100 => Vec3::from_values(*max_x, *max_y, *min_z),
                    0b101 => Vec3::from_values(*min_x, *max_y, *min_z),
                    0b110 => Vec3::from_values(*max_x, *min_y, *min_z),
                    0b111 => Vec3::from_values(*min_x, *min_y, *min_z),
                    _ => unreachable!(),
                },
            ),
            BoundingVolume::OrientedBoundingBox { center, x, y, z } => {
                let center = *center;
                let x = *x;
                let y = *y;
                let z = *z;
                cull_bb(planes, &center, |signs| match signs {
                    0b000 => center + x + y + z, // 000
                    0b001 => center + x + y - z, // 001
                    0b010 => center + x - y + z, // 010
                    0b011 => center + x - y - z, // 011
                    0b100 => center - x + y + z, // 100
                    0b101 => center - x + y - z, // 101
                    0b110 => center - x - y + z, // 110
                    0b111 => center - x - y - z, // 111
                    _ => unreachable!(),
                })
            }
        }
    }

    /// Transforms this bounding volume native by a transformation matrix.
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
            BoundingVolume::OrientedBoundingBox { center, x, y, z } => {
                #[rustfmt::skip]
                let before = Mat4::from_values(
                    x.0[0], y.0[0], z.0[0], center.0[0],
                    x.0[1], y.0[1], z.0[1], center.0[1],
                    x.0[2], y.0[2], z.0[2], center.0[2],
                    0.0, 0.0, 0.0, 1.0,
                );
                let after = *transformation * before;
                let x = Vec3::from_values(after.m00(), after.m10(), after.m20());
                let y = Vec3::from_values(after.m01(), after.m11(), after.m21());
                let z = Vec3::from_values(after.m02(), after.m12(), after.m22());
                let center = Vec3::from_values(after.m03(), after.m13(), after.m23());

                BoundingVolume::OrientedBoundingBox { center, x, y, z }
            }
        }
    }
}

/// Applies culling detection to a bounding sphere against a frustum
/// by calculating distances between sphere center to each plane of frustum.
fn cull_sphere(
    planes: [(FrustumPlaneIndex, Option<&Plane>); 6],
    center: &Vec3,
    radius: f64,
) -> Culling {
    let mut inside_count = 0u8;
    let mut distances = [None, None, None, None, None, None, None];

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
                    distances[*kind as usize] = Some((distance + radius).abs());
                // distance should always returns positive value
                } else {
                    // intersect, do nothing
                    distances[*kind as usize] = Some(distance.abs());
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
            top: distances[FrustumPlaneIndex::Top as usize].unwrap(),
            bottom: distances[FrustumPlaneIndex::Bottom as usize].unwrap(),
            left: distances[FrustumPlaneIndex::Left as usize].unwrap(),
            right: distances[FrustumPlaneIndex::Right as usize].unwrap(),
            near: distances[FrustumPlaneIndex::Near as usize].unwrap(),
            far: distances[FrustumPlaneIndex::Far as usize],
        }
    } else {
        Culling::Intersect {
            top: distances[FrustumPlaneIndex::Top as usize].unwrap(),
            bottom: distances[FrustumPlaneIndex::Bottom as usize].unwrap(),
            left: distances[FrustumPlaneIndex::Left as usize].unwrap(),
            right: distances[FrustumPlaneIndex::Right as usize].unwrap(),
            near: distances[FrustumPlaneIndex::Near as usize].unwrap(),
            far: distances[FrustumPlaneIndex::Far as usize],
        }
    }
}

/// [Optimized View Frustum Culling Algorithms for Bounding Boxes](https://www.cse.chalmers.se/~uffe/vfc_bbox.pdf)
/// For both AABB and OBB.
fn cull_bb<F: Fn(u8) -> Vec3>(
    planes: [(FrustumPlaneIndex, Option<&Plane>); 6],
    center: &Vec3,
    pnv_function: F,
) -> Culling {
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
                let pv = pnv_function(pi);
                let nv = pnv_function(ni);

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
            top: distances[FrustumPlaneIndex::Top as usize].unwrap(),
            bottom: distances[FrustumPlaneIndex::Bottom as usize].unwrap(),
            left: distances[FrustumPlaneIndex::Left as usize].unwrap(),
            right: distances[FrustumPlaneIndex::Right as usize].unwrap(),
            near: distances[FrustumPlaneIndex::Near as usize].unwrap(),
            far: distances[FrustumPlaneIndex::Far as usize],
        }
    } else {
        Culling::Inside {
            top: distances[FrustumPlaneIndex::Top as usize].unwrap(),
            bottom: distances[FrustumPlaneIndex::Bottom as usize].unwrap(),
            left: distances[FrustumPlaneIndex::Left as usize].unwrap(),
            right: distances[FrustumPlaneIndex::Right as usize].unwrap(),
            near: distances[FrustumPlaneIndex::Near as usize].unwrap(),
            far: distances[FrustumPlaneIndex::Far as usize],
        }
    }
}

/// Merges multiples [`BoundingVolumeRaw`]s into one [`BoundingVolumeRaw::BoundingSphere`].
/// Different situations will be handled differently:
/// - For a [`BoundingVolumeRaw::BoundingSphere`], merge
pub fn merge_bounding_volumes<'a, B>(boundings: B) -> Option<BoundingVolume>
where
    B: IntoIterator< Item = &'a BoundingVolume>,
{
    let boundings = boundings.into_iter();

    let mut output: Option<(Vec3, f64)> = None;

    for bounding in boundings {
        let (c2, r2) = match bounding {
            BoundingVolume::BoundingSphere { center, radius } => (*center, *radius),
            BoundingVolume::AxisAlignedBoundingBox {
                min_x,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
            } => {
                let center = bounding.center();
                let dx = (max_x - min_x).powi(2);
                let dy = (max_y - min_y).powi(2);
                let dz = (max_z - min_z).powi(2);
                let radius = (dx + dy + dz).sqrt() / 2.0;
                (center, radius)
            }
            BoundingVolume::OrientedBoundingBox { center, x, y, z } => (
                *center,
                (x.squared_length() + y.squared_length() + z.squared_length()).sqrt(),
            ),
        };

        match output {
            Some((c1, r1)) => {
                if c1 == c2 {
                    output = Some((c1, r1.max(r2)));
                } else {
                    // greater radius to be c1 and r1, the other to be c2 and r2
                    let (c1, r1, c2, r2) = if r1 > r2 {
                        (c1, r1, c2, r2)
                    } else {
                        (c2, r2, c1, r1)
                    };

                    let mut d = c2 - c1;
                    let l = d.length();

                    if r1 - l >= r2 {
                        // r2 completely inside r1
                        output = Some((c1, r1));
                    } else {
                        d = d.normalize();
                        let p2 = c1 + d * (l + r2);
                        let p1 = c2 - d * (l + r1);

                        let c = (p1 + p2) / 2.0;
                        let r = (l + r1 + r2) / 2.0;
                        output = Some((c, r));
                    }
                }
            }
            None => output = Some((c2, r2)),
        }
    }

    output.map(|(center, radius)| BoundingVolume::BoundingSphere { center, radius })
}

/// Culling status of a [`BoundingVolume`]\(or [`BoundingVolumeRaw`]\) against a [`ViewFrustum`]
/// with the shortest distance to each plane if inside or intersect.
///
/// Since far distance of a camera is optional,
/// shortest distance is optional for far plane as well.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Culling {
    Outside(FrustumPlaneIndex),
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
