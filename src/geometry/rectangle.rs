use std::any::Any;

use gl_matrix4rust::vec2::Vec2;

use crate::{
    bounding::BoundingVolume, clock::Tick, readonly::Readonly, renderer::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage, MemoryPolicy,
        },
        draw::{CullFace, Draw, DrawMode},
        uniform::{UniformBlockValue, UniformValue},
    }
};

use super::Geometry;

/// A 2-Dimensions plane on XY space
pub struct Rectangle {
    anchor: Vec2,
    placement: Placement,
    width: f64,
    height: f64,
    positions: AttributeValue,
    texture_coordinates: AttributeValue,
    normals: AttributeValue,
    tangents: AttributeValue,
    bitangents: AttributeValue,
    bounding: BoundingVolume,
}

impl Rectangle {
    pub fn new(
        anchor: Vec2,
        placement: Placement,
        width: f64,
        height: f64,
        texture_scale_s: f64,
        texture_scale_t: f64,
    ) -> Self {
        let (compositions, bounding) = create_rectangle(
            anchor,
            placement,
            width,
            height,
            texture_scale_s,
            texture_scale_t,
        );
        let descriptor = BufferDescriptor::with_memory_policy(
            BufferSource::from_binary(compositions, 0, compositions.len()),
            BufferUsage::STATIC_DRAW,
            MemoryPolicy::restorable(move || {
                let (compositions, _) = create_rectangle(
                    anchor,
                    placement,
                    width,
                    height,
                    texture_scale_s,
                    texture_scale_t,
                );
                BufferSource::from_binary(compositions, 0, compositions.len())
            }),
        );

        Self {
            anchor,
            placement,
            width,
            height,
            bounding,
            positions: AttributeValue::Buffer {
                descriptor: descriptor.clone(),
                target: BufferTarget::ARRAY_BUFFER,
                component_size: BufferComponentSize::Two,
                data_type: BufferDataType::FLOAT,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 0,
            },
            texture_coordinates: AttributeValue::Buffer {
                descriptor: descriptor.clone(),
                target: BufferTarget::ARRAY_BUFFER,
                component_size: BufferComponentSize::Two,
                data_type: BufferDataType::FLOAT,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 32,
            },
            normals: AttributeValue::Buffer {
                descriptor: descriptor.clone(),
                target: BufferTarget::ARRAY_BUFFER,
                component_size: BufferComponentSize::Three,
                data_type: BufferDataType::FLOAT,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 64,
            },
            tangents: AttributeValue::Buffer {
                descriptor: descriptor.clone(),
                target: BufferTarget::ARRAY_BUFFER,
                component_size: BufferComponentSize::Three,
                data_type: BufferDataType::FLOAT,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 112,
            },
            bitangents: AttributeValue::Buffer {
                descriptor,
                target: BufferTarget::ARRAY_BUFFER,
                component_size: BufferComponentSize::Three,
                data_type: BufferDataType::FLOAT,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 160,
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
            mode: DrawMode::TRIANGLE_FAN,
            first: 0,
            count: 4,
        }
    }

    fn cull_face(&self) -> Option<CullFace> {
        None
    }

    fn bounding_volume(&self) -> Option<Readonly<'_, BoundingVolume>> {
        Some(Readonly::Borrowed(&self.bounding))
    }

    fn positions(&self) -> Readonly<'_, AttributeValue> {
        Readonly::Borrowed(&self.positions)
    }

    fn normals(&self) -> Option<Readonly<'_, AttributeValue>> {
        Some(Readonly::Borrowed(&self.normals))
    }

    fn tangents(&self) -> Option<Readonly<'_, AttributeValue>> {
        Some(Readonly::Borrowed(&self.tangents))
    }

    fn bitangents(&self) -> Option<Readonly<'_, AttributeValue>> {
        Some(Readonly::Borrowed(&self.bitangents))
    }

    fn texture_coordinates(&self) -> Option<Readonly<'_, AttributeValue>> {
        Some(Readonly::Borrowed(&self.texture_coordinates))
    }

    fn attribute_value(&self, _: &str) -> Option<Readonly<'_, AttributeValue>> {
        None
    }

    fn uniform_value(&self, _: &str) -> Option<Readonly<'_, UniformValue>> {
        None
    }

    fn uniform_block_value(&self, _: &str) -> Option<Readonly<'_, UniformBlockValue>> {
        None
    }

    fn tick(&mut self, _: &Tick) -> bool {
        false
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
    texture_scale_s: f64,
    texture_scale_t: f64,
) -> ([u8; 208], BoundingVolume) {
    let x = *anchor.x();
    let y = *anchor.y();

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

    let bounding_volume = BoundingVolume::AxisAlignedBoundingBox {
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
        // positions
        max_x, min_y,
        max_x, max_y,
        min_x, max_y,
        min_x, min_y,
        // tex coordinates
        1.0 * texture_scale_s as f32, 0.0,
        1.0 * texture_scale_s as f32, 1.0 * texture_scale_t as f32,
        0.0, 1.0 * texture_scale_t as f32,
        0.0, 0.0,
        // normals
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
        // tangents
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        // bitangents
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
    ];

    (
        unsafe { std::mem::transmute::<[f32; 52], [u8; 208]>(buffer) },
        bounding_volume,
    )
}
