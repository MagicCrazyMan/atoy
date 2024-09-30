use std::{
    cell::{RefCell, RefMut},
    fmt::Debug,
    rc::Rc,
    vec::Drain,
};

use hashbrown::HashMap;
use tokio::sync::broadcast::{self, Receiver, Sender};
use uuid::Uuid;

/// Faces of cube map texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCubeMapFace {
    NegativeX,
    PositiveX,
    NegativeY,
    PositiveY,
    NegativeZ,
    PositiveZ,
}

pub trait TextureData {
    /// Converts the texture data into a [`WebGlTextureData`](super::web::webgl::texture::WebGlTextureData).
    #[cfg(feature = "webgl")]
    fn as_webgl_texture_data(&self) -> Option<super::web::webgl::texture::WebGlTextureData> {
        None
    }
}

pub(crate) struct TexturingItem {
    /// Texture data.
    pub(crate) data: Box<dyn TextureData>,
    pub(crate) cube_map_face: TextureCubeMapFace,
    pub(crate) dst_origin_x: Option<usize>,
    pub(crate) dst_origin_y: Option<usize>,
    pub(crate) dst_origin_z: Option<usize>,
    /// Width of the data to be replaced. If `None`, the width of the data is used.
    pub(crate) dst_width: Option<usize>,
    /// Height of the data to be replaced. If `None`, the height of the data is used.
    pub(crate) dst_height: Option<usize>,
    pub(crate) dst_depth_or_len: Option<usize>,
}

pub(crate) struct TexturingQueue {
    queue: Vec<TexturingItem>,
}

impl TexturingQueue {
    fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub(crate) fn drain(&mut self) -> Drain<'_, TexturingItem> {
        self.queue.drain(..)
    }
}

#[derive(Clone)]
pub struct Texturing {
    id: Uuid,
    /// Queue for each level.
    queues: Rc<RefCell<HashMap<usize, TexturingQueue>>>,

    channel: Sender<TexturingMessage>,
}

impl Texturing {
    /// Constructs a new texturing container.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            queues: Rc::new(RefCell::new(HashMap::new())),

            channel: broadcast::channel(5).0,
        }
    }

    /// Returns id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns the inner texturing queue.
    pub(crate) fn queue_of_level(&self, level: usize) -> RefMut<'_, TexturingQueue> {
        RefMut::map(self.queues.borrow_mut(), |queues| {
            queues.entry(level).or_insert_with(|| TexturingQueue::new())
        })
    }

    /// Pushes texture data into the texture.
    pub fn push<T>(&self, data: T, level: usize)
    where
        T: TextureData + 'static,
    {
        self.push_with_params(data, level, None, None, None, None, None, None, None)
    }

    /// Pushes texture data into the texture with byte offset indicating where to start replacing data.
    fn push_with_params<T>(
        &self,
        data: T,
        level: usize,
        cube_map_face: Option<TextureCubeMapFace>,
        dst_origin_x: Option<usize>,
        dst_origin_y: Option<usize>,
        dst_origin_z: Option<usize>,
        dst_width: Option<usize>,
        dst_height: Option<usize>,
        dst_depth_or_len: Option<usize>,
    ) where
        T: TextureData + 'static,
    {
        let mut queue = self.queue_of_level(level);
        let TexturingQueue { queue } = &mut *queue;

        let item = TexturingItem {
            data: Box::new(data),
            cube_map_face: cube_map_face.unwrap_or(TextureCubeMapFace::NegativeX),
            dst_origin_x,
            dst_origin_y,
            dst_origin_z,
            dst_width,
            dst_height,
            dst_depth_or_len,
        };
        queue.push(item);
    }

    /// Returns a message receiver associated with this texturing.
    pub fn receiver(&self) -> Receiver<TexturingMessage> {
        self.channel.subscribe()
    }
}

impl Debug for Texturing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texturing").field("id", self.id()).finish()
    }
}

impl Drop for Texturing {
    fn drop(&mut self) {
        let _ = self.channel.send(TexturingMessage::Dropped);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TexturingMessage {
    Dropped,
}
