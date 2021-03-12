use solstice::image::Settings;
use solstice::texture::{FilterMode, Texture, TextureType, TextureUpdate, WrapMode};
use solstice::PixelFormat;

pub struct Cache {
    pub(crate) texture: solstice::image::Image,
}

impl Cache {
    pub fn new(gl: &mut solstice::Context, width: u32, height: u32) -> Cache {
        let texture = solstice::image::Image::new(
            gl,
            TextureType::Tex2D,
            PixelFormat::Alpha,
            width,
            height,
            Settings {
                filter: FilterMode::Linear,
                wrap: WrapMode::Clamp,
                ..Default::default()
            },
        )
        .unwrap();

        gl.set_texture_data(
            texture.get_texture_key(),
            texture.get_texture_info(),
            texture.get_texture_type(),
            None,
        );

        Cache { texture }
    }

    pub unsafe fn update(
        &self,
        gl: &mut solstice::Context,
        offset: [u16; 2],
        size: [u16; 2],
        data: &[u8],
    ) {
        let [offset_x, offset_y] = offset;
        let [width, height] = size;

        let mut texture = self.texture.get_texture_info();
        texture.set_width(width as _);
        texture.set_height(height as _);
        gl.set_texture_sub_data(
            self.texture.get_texture_key(),
            texture,
            self.texture.get_texture_type(),
            data,
            offset_x as _,
            offset_y as _,
        );
    }

    pub unsafe fn destroy(&self, _gl: &mut solstice::Context) {
        // FIXME: we REALLY need cleanup function for the high level objects in solstice
    }
}
