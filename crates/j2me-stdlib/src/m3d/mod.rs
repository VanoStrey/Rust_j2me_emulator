// 3D API Stubs (Mascot Capsule v3 & JSR-184 M3G)
// Ensures 3D games load and initialize correctly without ClassNotFoundException or NoSuchMethodError

#[derive(Debug, Clone, Default)]
pub struct MascotFigure {
    pub id: i32,
    pub pattern: i32,
}

#[derive(Debug, Clone, Default)]
pub struct MascotActionTable {
    pub id: i32,
    pub num_actions: i32,
}

#[derive(Debug, Clone, Default)]
pub struct MascotGraphics3D {
    pub width: i32,
    pub height: i32,
}

impl MascotGraphics3D {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}
