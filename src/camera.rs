use crate::math::Vec3;

#[derive(Clone, Copy)]
pub struct Camera {
    pub eye: Vec3,
    pub center: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    fov: f32,
    changed: bool,
}

impl Camera {
    pub fn new(eye: Vec3, center: Vec3, up: Vec3, fov: f32) -> Self {
        let mut camera = Self {
            eye,
            center,
            up,
            forward: Vec3::default(),
            right: Vec3::default(),
            fov,
            changed: true,
        };
        camera.update_basis_vectors();
        camera
    }

    pub const fn origin(&self) -> Vec3 {
        self.eye
    }

    pub fn perspective_scale(&self) -> f32 {
        (self.fov * 0.5).tan()
    }

    pub fn update_basis_vectors(&mut self) {
        self.forward = (self.center - self.eye).normalize();
        self.right = self.forward.cross(self.up).normalize();
        self.up = self.right.cross(self.forward).normalize();
        self.changed = true;
    }

    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        let relative_position = self.eye - self.center;
        let radius = relative_position.length();

        if radius <= f32::EPSILON {
            return;
        }

        let current_yaw = relative_position.z.atan2(relative_position.x);
        let current_pitch = (relative_position.y / radius).asin();
        let new_yaw = current_yaw + yaw;
        let new_pitch = (current_pitch + pitch).clamp(-1.5, 1.5);
        let cos_pitch = new_pitch.cos();

        let new_relative_position = Vec3::new(
            radius * cos_pitch * new_yaw.cos(),
            radius * new_pitch.sin(),
            radius * cos_pitch * new_yaw.sin(),
        );

        self.eye = self.center + new_relative_position;
        self.update_basis_vectors();
    }

    pub fn zoom(&mut self, amount: f32) {
        let current_distance = (self.center - self.eye).length();
        let new_distance = (current_distance - amount).clamp(1.5, 20.0);
        let applied_amount = current_distance - new_distance;
        self.eye = self.eye + self.forward * applied_amount;
        self.update_basis_vectors();
    }

    #[allow(dead_code)]
    pub fn is_changed(&mut self) -> bool {
        let changed = self.changed;
        self.changed = false;
        changed
    }

    #[allow(dead_code)]
    pub fn basis_change(&self, vector: &Vec3) -> Vec3 {
        Vec3::new(
            vector.x * self.right.x + vector.y * self.up.x - vector.z * self.forward.x,
            vector.x * self.right.y + vector.y * self.up.y - vector.z * self.forward.y,
            vector.x * self.right.z + vector.y * self.up.z - vector.z * self.forward.z,
        )
    }

    #[allow(dead_code)]
    pub fn ray_direction(&self, x: usize, y: usize, width: usize, height: usize) -> Vec3 {
        let width = width as f32;
        let height = height as f32;
        let aspect_ratio = width / height;
        let perspective_scale = self.perspective_scale();

        // Se muestrea el centro del píxel y se invierte Y porque la imagen
        // comienza en la esquina superior izquierda.
        let screen_x = (2.0 * (x as f32 + 0.5)) / width - 1.0;
        let screen_y = 1.0 - (2.0 * (y as f32 + 0.5)) / height;
        let screen_x = screen_x * aspect_ratio * perspective_scale;
        let screen_y = screen_y * perspective_scale;

        let camera_direction = Vec3::new(screen_x, screen_y, -1.0);
        self.basis_change(&camera_direction).normalize()
    }
}
