use std::{
    cell::{RefCell, RefMut},
    fmt::Debug,
    rc::Rc,
    vec::Drain,
};

use hashbrown::HashMap;
use uuid::Uuid;

use super::channel::Channel;

pub trait TextureData {
    /// Returns width of the texture data.
    fn width(&self) -> usize;

    /// Returns height of the texture data.
    fn height(&self) -> usize;

    /// Returns width of the texture data.
    fn x_offset(&self) -> usize;

    /// Returns height of the texture data.
    fn y_offset(&self) -> usize;

    // /// Converts the texture data into a [`WebGlBufferData`](super::web::webgl::buffer::WebGlBufferData).
    // #[cfg(feature = "webgl")]
    // fn as_webgl_texture_data(&self) -> Option<super::web::webgl::texture::WebGlTex> {
    //     None
    // }
}

pub(crate) struct TexturingItem {
    pub(crate) data: Box<dyn TextureData>,
}

pub(crate) struct TexturingQueue {
    /// Queue for each level.
    queue: HashMap<usize, Vec<TexturingItem>>,
}

impl TexturingQueue {
    fn new() -> Self {
        Self {
            queue: HashMap::new(),
        }
    }

    pub(crate) fn drain_by_level(&mut self, level: usize) -> Option<Drain<'_, TexturingItem>> {
        self.queue.get_mut(&level).map(|q| q.drain(..))
    }
}

struct Managed {
    id: Uuid,
    channel: Channel,
}

#[derive(Clone)]
pub struct Texturing {
    id: Uuid,
    queue: Rc<RefCell<TexturingQueue>>,

    managed: Rc<RefCell<Option<Managed>>>,
}

impl Texturing {
    /// Constructs a new texturing container with default options.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            queue: Rc::new(RefCell::new(TexturingQueue::new())),

            managed: Rc::new(RefCell::new(None)),
        }
    }

    /// Returns id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns the inner texturing queue.
    pub(crate) fn queue(&self) -> RefMut<'_, TexturingQueue> {
        self.queue.borrow_mut()
    }

    /// Returns `true` if the texturing is managed.
    pub fn is_managed(&self) -> bool {
        self.managed.borrow().is_some()
    }

    /// Returns manager id.
    pub(crate) fn manager_id(&self) -> Option<Uuid> {
        self.managed.borrow().as_ref().map(|Managed { id, .. }| *id)
    }

    /// Sets this texturing is managed by a manager.
    pub(crate) fn set_managed(&self, id: Uuid, channel: Channel) {
        let mut managed = self.managed.borrow_mut();
        match managed.as_ref() {
            Some(managed) => {
                if managed.channel.id() != channel.id() || &managed.id != &id {
                    panic!("manage a texturing by multiple managers is prohibited");
                }
            }
            None => *managed = Some(Managed { id, channel }),
        };
    }

    // /// Pushes texture data into the texture.
    // pub fn push<T>(&self, data: T)
    // where
    //     T: TextureData + 'static,
    // {
    //     self.push_with_offset(data, 0)
    // }

    // /// Pushes texture data into the texture with byte offset indicating where to start replacing data.
    // pub fn push_with_offset<T>(&self, data: T, dst_byte_offset: usize)
    // where
    //     T: TextureData + 'static,
    // {
    //     let mut queue = self.queue.borrow_mut();
    //     let TextureQueue {
    //         queue,
    //         covered_byte_range,
    //     } = &mut *queue;
    //     let byte_length = dst_byte_offset + data.byte_length();
    //     let byte_range = dst_byte_offset..byte_length;
    //     self.byte_length
    //         .replace_with(|length| (*length).max(byte_length));

    //     let item = TextureItem {
    //         data: Box::new(data),
    //         dst_byte_offset,
    //     };

    //     match covered_byte_range {
    //         Some(covered_byte_range) => {
    //             // overrides queue if new byte range fully covers the range of current queue
    //             if byte_range.start <= covered_byte_range.start
    //                 && byte_range.end >= covered_byte_range.end
    //             {
    //                 *covered_byte_range = byte_range;
    //                 queue.clear();
    //                 queue.push(item);
    //             } else {
    //                 *covered_byte_range = byte_range.start.min(covered_byte_range.start)
    //                     ..byte_range.end.max(covered_byte_range.end);
    //                 queue.push(item);
    //             }
    //         }
    //         None => {
    //             *covered_byte_range = Some(byte_range);
    //             queue.push(item);
    //         }
    //     }
    // }
}

impl Debug for Texturing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texturing").field("id", self.id()).finish()
    }
}

impl Drop for Texturing {
    fn drop(&mut self) {
        if let Some(Managed { channel, .. }) = self.managed.borrow().as_ref() {
            channel.send(TexturingDropped { id: self.id });
        }
    }
}

/// Events raised when a [`Texturing`] is dropped.
pub(crate) struct TexturingDropped {
    id: Uuid,
}

impl TexturingDropped {
    /// Returns id.
    pub(crate) fn id(&self) -> &Uuid {
        &self.id
    }
}

// #[cfg(feature = "web")]
// impl TextureData for js_sys::ArrayBuffer {
//     fn byte_length(&self) -> usize {
//         self.byte_length() as usize
//     }

//     #[cfg(feature = "webgl")]
//     fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
//         Some(super::web::webgl::buffer::WebGlBufferData::ArrayBuffer { data: self })
//     }
// }

// macro_rules! web_typed_arrays {
//     ($(($buffer: ident, $length: ident, $size: expr))+) => {
//         $(
//             #[cfg(feature = "web")]
//             impl BufferData for js_sys::$buffer {
//                 fn byte_length(&self) -> usize {
//                     self.byte_length() as usize
//                 }

//                 #[cfg(feature = "webgl")]
//                 fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
//                     Some(super::web::webgl::buffer::WebGlBufferData::$buffer { data: self, element_range: None })
//                 }
//             }

//             #[cfg(feature = "webgl")]
//             impl BufferData for (js_sys::$buffer, super::web::webgl::buffer::WebGlBufferDataRange) {
//                 fn byte_length(&self) -> usize {
//                     let data_element_length = self.0.$length() as usize;
//                     let element_length = match &self.1 {
//                         super::web::webgl::buffer::WebGlBufferDataRange::Range(range) => {
//                             if range.start > data_element_length {
//                                 0
//                             } else if range.end > data_element_length {
//                                 data_element_length - range.start
//                             } else {
//                                 range.len()
//                             }
//                         },
//                         super::web::webgl::buffer::WebGlBufferDataRange::RangeFrom(range) => {
//                             if range.start > data_element_length {
//                                 0
//                             } else {
//                                 data_element_length - range.start
//                             }
//                         },
//                     };

//                     element_length * $size
//                 }

//                 fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
//                     Some(super::web::webgl::buffer::WebGlBufferData::$buffer { data: &self.0, element_range: Some(self.1.clone()) })
//                 }
//             }
//         )+
//     };
// }
// web_typed_arrays! {
//     (DataView, byte_length, 1)
//     (Int8Array, length, 1)
//     (Uint8Array, length, 1)
//     (Uint8ClampedArray, length, 1)
//     (Int16Array, length, 2)
//     (Uint16Array, length, 2)
//     (Int32Array, length, 4)
//     (Uint32Array, length, 4)
//     (Float32Array, length, 4)
//     (Float64Array, length, 8)
//     (BigInt64Array, length, 8)
//     (BigUint64Array, length, 8)
// }
