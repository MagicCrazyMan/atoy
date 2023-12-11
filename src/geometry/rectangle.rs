use std::any::Any;

use gl_matrix4rust::vec2::Vec2;

use crate::{
    bounding::BoundingVolumeNative,
    entity::BorrowedMut,
    render::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage, MemoryPolicy,
        },
        draw::{Draw, DrawMode},
        uniform::UniformValue,
    },
};

use super::Geometry;

/// A 2-Dimensions plane on XY space
pub struct Rectangle {
    anchor: Vec2,
    placement: Placement,
    width: f64,
    height: f64,
    vertices: AttributeValue,
    texture_coordinates: AttributeValue,
    normal: AttributeValue,
    bounding: BoundingVolumeNative,
}

impl Rectangle {
    pub fn new(anchor: Vec2, placement: Placement, width: f64, height: f64) -> Self {
        let (compositions, bounding) = create_rectangle(anchor, placement, width, height);
        let share_descriptor = BufferDescriptor::with_memory_policy(
            BufferSource::from_binary(compositions, 0, compositions.len() as u32),
            BufferUsage::StaticDraw,
            MemoryPolicy::from_restorable(move || {
                let composition = create_rectangle(anchor, placement, width, height).0;
                BufferSource::from_binary(composition, 0, composition.len() as u32)
            }),
        );

        Self {
            anchor,
            placement,
            width,
            height,
            bounding,
            vertices: AttributeValue::Buffer {
                descriptor: share_descriptor.clone(),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Two,
                data_type: BufferDataType::Float,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 0,
            },
            texture_coordinates: AttributeValue::Buffer {
                descriptor: share_descriptor.clone(),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Two,
                data_type: BufferDataType::Float,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: (4 * 2) * 4,
            },
            normal: AttributeValue::Buffer {
                descriptor: share_descriptor.clone(),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Three,
                data_type: BufferDataType::Float,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: (4 * 2 + 4 * 2) * 4,
            },
        }
    }

    pub fn anchor(&self) -> Vec2<f64> {
        self.anchor
    }

    pub fn placement(&self) -> Placement {
        self.placement
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }
}

impl Geometry for Rectangle {
    fn draw(&self) -> Draw {
        Draw::Arrays {
            mode: DrawMode::TriangleFan,
            first: 0,
            count: 4,
        }
    }

    fn bounding_volume_native(&self) -> Option<BoundingVolumeNative> {
        Some(self.bounding)
    }

    fn vertices(&self) -> Option<AttributeValue> {
        Some(self.vertices.clone())
    }

    fn normals(&self) -> Option<AttributeValue> {
        Some(self.normal.clone())
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        Some(self.texture_coordinates.clone())
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str, _: &BorrowedMut) -> Option<UniformValue> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Placement {
    Center,
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
    TopCenter,
    RightCenter,
    BottomCenter,
    LeftCenter,
}

fn create_rectangle(
    anchor: Vec2,
    placement: Placement,
    width: f64,
    height: f64,
) -> ([u8; 112], BoundingVolumeNative) {
    let x = anchor.0[0];
    let y = anchor.0[1];

    let (min_x, max_x, min_y, max_y) = match placement {
        Placement::Center => {
            let half_width = width / 2.0;
            let half_height = height / 2.0;

            (
                x - half_width,
                x + half_width,
                y - half_height,
                y + half_height,
            )
        }
        Placement::TopLeft => (x, x + width, y - height, y),
        Placement::TopRight => (x - width, x, y - height, y),
        Placement::BottomRight => (x - width, x, y, y + height),
        Placement::BottomLeft => (x, x + width, y, y + height),
        Placement::TopCenter => {
            let half_width = width / 2.0;
            (x - half_width, x + half_width, y - height, y)
        }
        Placement::RightCenter => {
            let half_height = height / 2.0;
            (x - width, x, y - half_height, y + half_height)
        }
        Placement::BottomCenter => {
            let half_width = width / 2.0;
            (x - half_width, x + half_width, y, y + height)
        }
        Placement::LeftCenter => {
            let half_height = height / 2.0;
            (x, x + width, y - half_height, y + half_height)
        }
    };

    let bounding_volume = BoundingVolumeNative::AxisAlignedBoundingBox {
        min_x,
        max_x,
        min_y,
        max_y,
        min_z: -0.01,
        max_z: 0.01,
    };

    let (min_x, max_x, min_y, max_y) = (min_x as f32, max_x as f32, min_y as f32, max_y as f32);
    #[rustfmt::skip]
    let buffer = [
        // vertices
        max_x, min_y,  max_x, max_y,  min_x, max_y,  min_x, min_y,
        // tex coordinates
        1.0, 0.0,  1.0, 1.0,  0.0, 1.0,  0.0, 0.0,
        // normal
        0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0
    ];

    (
        unsafe {
            std::mem::transmute::<[f32; 28], [u8; 112]>(
                buffer,
            )
        },
        bounding_volume,
    )
}
