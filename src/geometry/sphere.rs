use std::any::Any;

use gl_matrix4rust::vec3::{AsVec3, Vec3};
use web_sys::js_sys::Float32Array;

use crate::{
    bounding::BoundingVolumeNative,
    render::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage,
        },
        draw::{Draw, DrawMode},
        uniform::UniformValue,
    },
};

use super::{Geometry, GeometryRenderEntity};

pub struct Sphere {
    radius: f64,
    vertical_segments: usize,
    horizontal_segments: usize,
    num_vertices: usize,
    vertices: BufferDescriptor,
    normals: BufferDescriptor,
    // non-clone fields
    update_bounding_volume: bool,
}

impl Sphere {
    pub fn new() -> Sphere {
        Self::with_opts(1.0, 12, 24)
    }

    pub fn with_opts(radius: f64, vertical_segments: usize, horizontal_segments: usize) -> Sphere {
        let (num_vertices, vertices, normals) =
            calculate_vertices_normals(radius, vertical_segments, horizontal_segments);
        let vertices_len = vertices.length();
        let normals_len = normals.length();
        Self {
            radius,
            vertical_segments,
            horizontal_segments,
            num_vertices,
            vertices: BufferDescriptor::new(
                BufferSource::from_float32_array(vertices, 0, vertices_len),
                BufferUsage::StaticDraw,
            ),
            normals: BufferDescriptor::new(
                BufferSource::from_float32_array(normals, 0, normals_len),
                BufferUsage::StaticDraw,
            ),
            update_bounding_volume: true,
        }
    }
}

impl Sphere {
    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn set_radius(&mut self, radius: f64) {
        self.radius = radius;
        let (num_vertices, vertices, normals) =
            calculate_vertices_normals(radius, self.vertical_segments, self.horizontal_segments);
        let vertices_len = vertices.length();
        let normals_len = normals.length();

        self.num_vertices = num_vertices;
        self.vertices
            .buffer_sub_data(BufferSource::from_float32_array(vertices, 0, vertices_len));
        self.normals
            .buffer_sub_data(BufferSource::from_float32_array(normals, 0, normals_len));
        self.update_bounding_volume = true;
    }
}

impl Geometry for Sphere {
    fn draw(&self) -> Draw {
        Draw::Arrays {
            mode: DrawMode::Triangles,
            first: 0,
            count: self.num_vertices as i32,
        }
    }

    fn bounding_volume_native(&self) -> Option<BoundingVolumeNative> {
        Some(BoundingVolumeNative::BoundingSphere {
            center: Vec3::from_values(0.0, 0.0, 0.0),
            radius: self.radius,
        })
    }

    fn vertices(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.vertices.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn normals(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.normals.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Four,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        None
    }

    fn attribute_value(&self, _: &str, _: &GeometryRenderEntity) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str, _: &GeometryRenderEntity) -> Option<UniformValue> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn update_bounding_volume(&self) -> bool {
        self.update_bounding_volume
    }

    fn set_update_bounding_volume(&mut self, v: bool) {
        self.update_bounding_volume = v;
    }
}

impl Clone for Sphere {
    fn clone(&self) -> Self {
        Self {
            radius: self.radius.clone(),
            vertical_segments: self.vertical_segments.clone(),
            horizontal_segments: self.horizontal_segments.clone(),
            num_vertices: self.num_vertices.clone(),
            vertices: self.vertices.clone(),
            normals: self.normals.clone(),
            update_bounding_volume: true,
        }
    }
}

#[rustfmt::skip]
fn calculate_vertices_normals(radius: f64, vertical_segments: usize, horizontal_segments: usize) -> (usize, Float32Array, Float32Array) {
    let vertical_offset = std::f64::consts::PI / vertical_segments as f64;
    let horizontal_offset = (2.0 * std::f64::consts::PI) / horizontal_segments as f64;
  
    let mut vertices = vec![0.0f32; (vertical_segments + 1) * (horizontal_segments + 1) * 3];
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
        vertices[(i * (horizontal_segments + 1) + j) * 3 + 0] = x as f32;
        vertices[(i * (horizontal_segments + 1) + j) * 3 + 1] = y as f32;
        vertices[(i * (horizontal_segments + 1) + j) * 3 + 2] = z as f32;
      }
    }
  
    let mut triangle_vertices = vec![0.0f32; vertical_segments * horizontal_segments * 2 * 3 * 3];
    let mut triangle_normals = vec![0.0f32; vertical_segments * horizontal_segments * 2 * 3 * 4];
    for i in 0..vertical_segments {
      for j in 0..horizontal_segments {
        let index0 = i * (horizontal_segments + 1) + j;
        let index1 = index0 + (horizontal_segments + 1);
        let index2 = index1 + 1;
        let index3 = index0 + 1;
  
        let vertex0 = &vertices[index0 * 3 + 0..index0 * 3 + 3];
        let vertex1 = &vertices[index1 * 3 + 0..index1 * 3 + 3];
        let vertex2 = &vertices[index2 * 3 + 0..index2 * 3 + 3];
        let vertex3 = &vertices[index3 * 3 + 0..index3 * 3 + 3];
  
        let d0 = (vertex0[0].powf(2.0) + vertex0[1].powf(2.0) + vertex0[2].powf(2.0)).sqrt();
        let normal0 = [vertex0[0] / d0, vertex0[1] / d0, vertex0[2] / d0, 0.0];
        let d1 = (vertex1[0].powf(2.0) + vertex1[1].powf(2.0) + vertex1[2].powf(2.0)).sqrt();
        let normal1 = [vertex1[0] / d1, vertex1[1] / d1, vertex1[2] / d1, 0.0];
        let d2 = (vertex2[0].powf(2.0) + vertex2[1].powf(2.0) + vertex2[2].powf(2.0)).sqrt();
        let normal2 = [vertex2[0] / d2, vertex2[1] / d2, vertex2[2] / d2, 0.0];
        let d3 = (vertex3[0].powf(2.0) + vertex3[1].powf(2.0) + vertex3[2].powf(2.0)).sqrt();
        let normal3 = [vertex3[0] / d3, vertex3[1] / d3, vertex3[2] / d3, 0.0];
  
        let start_index = (i * horizontal_segments + j) * 18 + 0;
        triangle_vertices.splice(start_index..start_index + vertex0.len(), vertex0.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 3;
        triangle_vertices.splice(start_index..start_index + vertex0.len(), vertex2.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 6;
        triangle_vertices.splice(start_index..start_index + vertex0.len(), vertex1.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 9;
        triangle_vertices.splice(start_index..start_index + vertex0.len(), vertex0.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 12;
        triangle_vertices.splice(start_index..start_index + vertex0.len(), vertex3.iter().cloned());
        let start_index = (i * horizontal_segments + j) * 18 + 15;
        triangle_vertices.splice(start_index..start_index + vertex0.len(), vertex2.iter().cloned());
  
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

    let vertices = Float32Array::new_with_length(triangle_vertices.len() as u32);
    let normals = Float32Array::new_with_length(triangle_normals.len() as u32);
    vertices.copy_from(&triangle_vertices);
    normals.copy_from(&triangle_normals);
    (
        (triangle_vertices.len() / 3),
        vertices,
        normals,
    )
}
