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
    // /// Returns width of the texture data.
    // fn width(&self) -> usize;

    // /// Returns height of the texture data.
    // fn height(&self) -> usize;

    // /// Returns x origin starts to copy image data of the texture data.
    // fn x_origin(&self) -> usize;

    // /// Returns y origin starts to copy image data of the texture data.
    // fn y_origin(&self) -> usize;

    /// Converts the texture data into a [`WebGlTextureData`](super::web::webgl::texture::WebGlTextureData).
    #[cfg(feature = "webgl")]
    fn as_webgl_texture_data(&self) -> Option<super::web::webgl::texture::WebGlTextureData> {
        None
    }
}

pub(crate) struct TexturingItem {
    pub(crate) data: Box<dyn TextureData>,
    pub(crate) level: usize,
    pub(crate) dst_origin_x: usize,
    pub(crate) dst_origin_y: usize,
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
    /// Queue for each level.
    queues: Rc<RefCell<HashMap<usize, TexturingQueue>>>,

    managed: Rc<RefCell<Option<Managed>>>,
}

impl Texturing {
    /// Constructs a new texturing container.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            queues: Rc::new(RefCell::new(HashMap::new())),

            managed: Rc::new(RefCell::new(None)),
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
        self.push_with_origin(data, level, 0, 0)
    }

    /// Pushes texture data into the texture with byte offset indicating where to start replacing data.
    pub fn push_with_origin<T>(
        &self,
        data: T,
        level: usize,
        dst_origin_x: usize,
        dst_origin_y: usize,
    ) where
        T: TextureData + 'static,
    {
        let mut queue = self.queue_of_level(level);
        let TexturingQueue { queue } = &mut *queue;

        let item = TexturingItem {
            data: Box::new(data),
            level,
            dst_origin_x,
            dst_origin_y,
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
