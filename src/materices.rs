use gl_matrix4rust::{error::Error, mat4::Mat4};

pub struct EntityMatrices {
    model: Mat4,
    n: Mat4,
    m: Mat4,
    mv: Mat4,
    mvp: Mat4,
}

impl EntityMatrices {
    pub fn new() -> EntityMatrices {
        Self {
            model: Mat4::new_identity(),
            n: Mat4::new_identity(),
            m: Mat4::new_identity(),
            mv: Mat4::new_identity(),
            mvp: Mat4::new_identity(),
        }
    }

    pub fn with_model_matrix(model: Mat4) -> EntityMatrices {
        Self {
            model,
            n: Mat4::new_identity(),
            m: Mat4::new_identity(),
            mv: Mat4::new_identity(),
            mvp: Mat4::new_identity(),
        }
    }
}

impl EntityMatrices {
    pub fn model(&self) -> &Mat4 {
        &self.model
    }

    pub fn composed_normal(&self) -> &Mat4 {
        &self.n
    }

    pub fn composed_model(&self) -> &Mat4 {
        &self.m
    }

    pub fn composed_model_view(&self) -> &Mat4 {
        &self.mv
    }

    pub fn composed_model_view_proj(&self) -> &Mat4 {
        &self.mvp
    }

    pub fn set_model(&mut self, mat: Mat4) {
        self.model = mat;
    }

    pub fn set_composed_model(&mut self, mat: Mat4) -> Result<(), Error> {
        self.n = mat.invert()?.transpose();
        self.m = mat;

        Ok(())
    }

    pub fn set_composed_model_view(&mut self, mat: Mat4) {
        self.mv = mat;
    }

    pub fn set_composed_model_view_proj(&mut self, mat: Mat4) {
        self.mvp = mat;
    }
}
