use std::{
    cell::{RefCell, RefMut},
    fmt::Debug,
    rc::Rc,
    vec::Drain,
};

use hashbrown::HashMap;
use uuid::Uuid;

use super::channel::Channel;

/// Texture dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDimension {
    One {
        width: usize,
    },
    Two {
        width: usize,
        height: usize,
    },
    Three {
        width: usize,
        height: usize,
        depth: usize,
    },
    CubeMap {
        width: usize,
        height: usize,
    },
}

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
    pub(crate) dst_origin_x: usize,
    pub(crate) dst_origin_y: usize,
    pub(crate) dst_origin_z: usize,
    /// Width of the data to be replaced. If `None`, the width of the data is used.
    pub(crate) dst_width: Option<usize>,
    /// Height of the data to be replaced. If `None`, the height of the data is used.
    pub(crate) dst_height: Option<usize>,
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

struct Managed {
    id: Uuid,
    channel: Channel,
}

#[derive(Clone)]
pub struct Texturing {
    id: Uuid,
    dimension: TextureDimension,
    array_len: Option<usize>,
    /// Queue for each level.
    queues: Rc<RefCell<HashMap<usize, TexturingQueue>>>,

    managed: Rc<RefCell<Option<Managed>>>,
}

impl Texturing {
    /// Constructs a new texturing container.
    pub fn new(dimension: TextureDimension) -> Self {
        Self {
            id: Uuid::new_v4(),
            dimension,
            array_len: None,
            queues: Rc::new(RefCell::new(HashMap::new())),

            managed: Rc::new(RefCell::new(None)),
        }
    }

    /// Constructs a new texturing array container.
    /// Converts to `None` if array length is `0`.
    pub fn new_array(dimension: TextureDimension, array_len: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            dimension,
            array_len: if array_len == 0 { None } else { Some(array_len) },
            queues: Rc::new(RefCell::new(HashMap::new())),

            managed: Rc::new(RefCell::new(None)),
        }
    }

    /// Returns id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns texture dimension.
    pub fn dimension(&self) -> TextureDimension {
        self.dimension
    }
    
    /// Returns length of the array if this texture is a texture array.
    /// Returns `None` if this texture is not an array.
    pub fn array_len(&self) -> Option<usize> {
        self.array_len
    }

    /// Returns the inner texturing queue.
    pub(crate) fn queue_of_level(&self, level: usize) -> RefMut<'_, TexturingQueue> {
        RefMut::map(self.queues.borrow_mut(), |queues| {
            queues.entry(level).or_insert_with(|| TexturingQueue::new())
        })
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

    /// Pushes texture data into the texture.
    pub fn push<T>(&self, data: T, level: usize)
    where
        T: TextureData + 'static,
    {
        self.push_with_params(data, level, 0, 0, 0, None, None)
    }

    /// Pushes texture data into the texture with byte offset indicating where to start replacing data.
    pub fn push_with_params<T>(
        &self,
        data: T,
        level: usize,
        dst_origin_x: usize,
        dst_origin_y: usize,
        dst_origin_z: usize,
        dst_width: Option<usize>,
        dst_height: Option<usize>,
    ) where
        T: TextureData + 'static,
    {
        let mut queue = self.queue_of_level(level);
        let TexturingQueue { queue } = &mut *queue;

        let item = TexturingItem {
            data: Box::new(data),
            dst_origin_x,
            dst_origin_y,
            dst_origin_z,
            dst_width,
            dst_height,
        };
        queue.push(item);
    }
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
