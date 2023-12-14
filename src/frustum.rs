use crate::plane::Plane;

#[derive(Debug, Clone, Copy)]
pub struct ViewFrustum {
    left: Plane,
    right: Plane,
    top: Plane,
    bottom: Plane,
    near: Plane,
    far: Option<Plane>,
}

impl ViewFrustum {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FrustumPlaneIndex {
    Top = 0,
    Bottom = 1,
    Left = 2,
    Right = 3,
    Near = 4,
    Far = 5,
}
