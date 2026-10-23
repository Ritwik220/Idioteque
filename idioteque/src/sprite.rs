
pub struct Sprite {
    pub texture: texture::Texture,
    pub x: f32,
    pub y: f32,
    pub height: f32,
    pub width: f32
} 




impl Sprite {
    fn create(texture: texture::Texture, x: f32, y:f32, height: f32, width: f32) -> Sprite {
        Sprite {
            texture,
            x,
            y,
            height,
            width
        }
    }
}

