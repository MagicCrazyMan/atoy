use crate::plane::Plane;

/// View frustum, for a view frustum, far plane is optional.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewFrustum {
    left: Plane,
    right: Plane,
    top: Plane,
    bottom: Plane,
    near: Plane,
    far: Option<Plane>,
}

impl ViewFrustum {
    /// Constructs a new view frustum.
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

    /// Returns the near plan.
    pub fn near(&self) -> &Plane {
        &self.near
    }

    /// Returns the far plan, optional.
    pub fn far(&self) -> Option<&Plane> {
        self.far.as_ref()
    }

    /// Returns the left plan.
    pub fn left(&self) -> &Plane {
        &self.left
    }

    /// Returns the right plan.
    pub fn right(&self) -> &Plane {
        &self.right
    }

    /// Returns the top plan.
    pub fn top(&self) -> &Plane {
        &self.top
    }

    /// Returns the bottom plan.
    pub fn bottom(&self) -> &Plane {
        &self.bottom
    }
}

/// Numerical indices of each plane.
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
