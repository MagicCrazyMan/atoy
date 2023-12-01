use crate::plane::Plane;

#[derive(Debug, Clone, Copy)]
pub struct ViewingFrustum {
    left: Plane,
    right: Plane,
    top: Plane,
    bottom: Plane,
    near: Plane,
    far: Option<Plane>,
}

impl ViewingFrustum {
    pub fn new(
        left: Plane,
        right: Plane,
        top: Plane,
        bottom: Plane,
        near: Plane,
        far: Option<Plane>,
    ) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
            near,
            far,
        }
    }

    pub fn near(&self) -> &Plane {
        &self.near
    }

    pub fn far(&self) -> Option<&Plane> {
        self.far.as_ref()
    }

    pub fn left(&self) -> &Plane {
        &self.left
    }

    pub fn right(&self) -> &Plane {
        &self.right
    }

    pub fn top(&self) -> &Plane {
        &self.top
    }

    pub fn bottom(&self) -> &Plane {
        &self.bottom
    }
}
