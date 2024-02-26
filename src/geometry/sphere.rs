use std::any::Any;

use gl_matrix4rust::vec3::Vec3;
use web_sys::js_sys::Float32Array;

use crate::{
    bounding::BoundingVolume, clock::Tick, readonly::Readonly, renderer::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage,
        },
        draw::{CullFace, Draw, DrawMode},
        uniform::{UniformBlockValue, UniformValue},
    }
};

use super::Geometry;

pub struct Sphere {
    radius: f64,
    vertical_segments: usize,
    horizontal_segments: usize,
    num_positions: usize,
    positions: BufferDescriptor,
    normals: BufferDescriptor,
    bounding_volume: BoundingVolume,
}

impl Sphere {
    pub fn new() -> Sphere {
        Self::with_params(1.0, 12, 24)
    }

    pub fn with_params(
        radius: f64,
        vertical_segments: usize,
        horizontal_segments: usize,
    ) -> Sphere {
        let (num_positions, positions, normals) =
            build_positions_and_normals(radius, vertical_segments, horizontal_segments);
        let positions_len = positions.length() as usize;
        let normals_len = normals.length() as usize;
        Self {
            radius,
            vertical_segments,
            horizontal_segments,
            num_positions,
            positions: BufferDescriptor::new(
                BufferSource::from_float32_array(positions, 0, positions_len),
                BufferUsage::STATIC_DRAW,
            ),
            normals: BufferDescriptor::new(
                BufferSource::from_float32_array(normals, 0, normals_len),
                BufferUsage::STATIC_DRAW,
            ),
            bounding_volume: BoundingVolume::BoundingSphere {
                center: Vec3::<f64>::new(0.0, 0.0, 0.0),
                radius,
            },
        }
    }
}

impl Sphere {
    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn set_radius(&mut self, radius: f64) {
        self.radius = radius;
        let (num_positions, positions, normals) =
            build_positions_and_normals(radius, self.vertical_segments, self.horizontal_segments);
        let positions_len = positions.length() as usize;
        let normals_len = normals.length() as usize;

        self.num_positions = num_positions;
        self.positions.buffer_sub_data(
            BufferSource::from_float32_array(positions, 0, positions_len),
            0,
        );
        self.normals
            .buffer_sub_data(BufferSource::from_float32_array(normals, 0, normals_len), 0);
        self.bounding_volume = BoundingVolume::BoundingSphere {
            center: Vec3::<f64>::new(0.0, 0.0, 0.0),
            radius,
        };
    }
}

impl Geometry for Sphere {
    fn draw(&self) -> Draw {
        Draw::Arrays {
            mode: DrawMode::TRIANGLES,
            first: 0,
            count: self.num_positions as i32,
        }
    }

    fn cull_face(&self) -> Option<CullFace> {
        Some(CullFace::BACK)
    }

    fn bounding_volume(&self) -> Option<Readonly<'_, BoundingVolume>> {
        Some(Readonly::Borrowed(&self.bounding_volume))
    }

    fn positions(&self) -> Readonly<'_, AttributeValue> {
        Readonly::Owned(AttributeValue::Buffer {
            descriptor: self.positions.clone(),
            target: BufferTarget::ARRAY_BUFFER,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn normals(&self) -> Option<Readonly<'_, AttributeValue>> {
        Some(Readonly::Owned(AttributeValue::Buffer {
            descriptor: self.normals.clone(),
            target: BufferTarget::ARRAY_BUFFER,
            component_size: BufferComponentSize::Four,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        }))
    }

    fn tangents(&self) -> Option<Readonly<'_, AttributeValue>> {
        None
    }

    fn bitangents(&self) -> Option<Readonly<'_, AttributeValue>> {
        None
    }

    fn texture_coordinates(&self) -> Option<Readonly<'_, AttributeValue>> {
        None
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

#[rustfmt::skip]
fn build_positions_and_normals(radius: f64, vertical_segments: usize, horizontal_segments: usize) -> (usize, Float32Array, Float32Array) {
    let vertical_offset = std::f64::consts::PI / vertical_segments as f64;
    let horizontal_offset = (2.0 * std::f64::consts::PI) / horizontal_segments as f64;
  
    let mut positions = vec![0.0f32; (vertical_segments + 1) * (horizontal_segments + 1) * 3];
    for i in 0..=vertical_segments {
      let ri = i as f64 * vertical_offset;
      let ci = ri.cos();
      let si = ri.sin();
  
      for j in 0..=horizontal_segments {
        let rj = j as f64 * horizontal_offset;
        let cj = rj.cos();
        let sj = rj.sin();
  
        let x = radius * si * cj;
        let y = radius * ci;
        let z = radius * si * sj;
        positions[(i * (horizontal_segments + 1) + j) * 3 + 0] = x as f32;
        positions[(i * (horizontal_segments + 1) + j) * 3 + 1] = y as f32;
        positions[(i * (horizontal_segments + 1) + j) * 3 + 2] = z as f32;
      }
    }
  
    let mut triangle_positions = vec![0.0f32; vertical_segments * horizontal_segments * 2 * 3 * 3];
    let mut triangle_normals = vec![0.0f32; vertical_segments * horizontal_segments * 2 * 3 * 4];
    for i in 0..vertical_segments {
      for j in 0..horizontal_segments {
        let index0 = i * (horizontal_segments + 1) + j;
        let index1 = index0 + (horizontal_segments + 1);
        let index2 = index1 + 1;
        let index3 = index0 + 1;
  
        let vertex0 = &positions[index0 * 3 + 0..index0 * 3 + 3];
        let vertex1 = &positions[index1 * 3 + 0..index1 * 3 + 3];
        let vertex2 = &positions[index2 * 3 + 0..index2 * 3 + 3];
        let vertex3 = &positions[index3 * 3 + 0..index3 * 3 + 3];
  
        let d0 = (vertex0[0].powf(2.0) + vertex0[1].powf(2.0) + vertex0[2].powf(2.0)).sqrt();
        let normal0 = [vertex0[0] / d0, vertex0[1] / d0, vertex0[2] / d0, 0.0];
        let d1 = (vertex1[0].powf(2.0) + vertex1[1].powf(2.0) + vertex1[2].powf(2.0)).sqrt();
        let normal1 = [vertex1[0] / d1, vertex1[1] / d1, vertex1[2] / d1, 0.0];
        let d2 = (vertex2[0].powf(2.0) + vertex2[1].powf(2.0) + vertex2[2].powf(2.0)).sqrt();
        let normal2 = [vertex2[0] / d2, vertex2[1] / d2, vertex2[2] / d2, 0.0];
        let d3 = (vertex3[0].powf(2.0) + vertex3[1].powf(2.0) + vertex3[2].powf(2.0)).sqrt();
        let normal3 = [vertex3[0] / d3, vertex3[1] / d3, vertex3[2] / d3, 0.0];
  
        let start_index = (i * horizontal_segments + j) * 18 + 0;
        triangle_positions.splice(start_index..start_index + vertex0.len(), vertex0.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 3;
        triangle_positions.splice(start_index..start_index + vertex0.len(), vertex2.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 6;
        triangle_positions.splice(start_index..start_index + vertex0.len(), vertex1.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 9;
        triangle_positions.splice(start_index..start_index + vertex0.len(), vertex0.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 12;
        triangle_positions.splice(start_index..start_index + vertex0.len(), vertex3.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 15;
        triangle_positions.splice(start_index..start_index + vertex0.len(), vertex2.iter().cloned());
  
        let start_index = (i * horizontal_segments + j) * 24 + 0;
        triangle_normals.splice(start_index..start_index + vertex0.len(), normal0.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 24 + 4;
        triangle_normals.splice(start_index..start_index + vertex0.len(), normal2.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 24 + 8;
        triangle_normals.splice(start_index..start_index + vertex0.len(), normal1.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 24 + 12;
        triangle_normals.splice(start_index..start_index + vertex0.len(), normal0.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 24 + 16;
        triangle_normals.splice(start_index..start_index + vertex0.len(), normal3.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 24 + 20;
        triangle_normals.splice(start_index..start_index + vertex0.len(), normal2.iter().cloned());
      }
    }

    let positions = Float32Array::new_with_length(triangle_positions.len() as u32);
    let normals = Float32Array::new_with_length(triangle_normals.len() as u32);
    positions.copy_from(&triangle_positions);
    normals.copy_from(&triangle_normals);
    (
        (triangle_positions.len() / 3),
        positions,
        normals,
    )
}
