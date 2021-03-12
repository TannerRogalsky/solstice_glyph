mod cache;

use crate::ab_glyph::point;
use crate::Region;
use cache::Cache;
use solstice::quad_batch::Quad;
use solstice::texture::Texture;
use solstice::{
    quad_batch::QuadBatch,
    shader::DynamicShader,
    shader::{RawUniformValue, UniformLocation},
    vertex::Vertex,
    PipelineSettings,
};

pub struct Pipeline {
    program: DynamicShader,
    instances: QuadBatch<Vertex2D>,
    cache: Cache,
    transform: UniformLocation,
    current_transform: [f32; 16],
}

impl Pipeline {
    pub fn new(gl: &mut solstice::Context, cache_width: u32, cache_height: u32) -> Pipeline {
        let cache = Cache::new(gl, cache_width, cache_height);

        let program = {
            const SRC: &str = include_str!("shader.glsl");
            let (vert, frag) = DynamicShader::create_source(SRC, SRC);
            DynamicShader::new(gl, vert.as_str(), frag.as_str()).unwrap()
        };

        let instances = QuadBatch::new(gl, Vertex2D::INITIAL_AMOUNT).unwrap();

        let transform = program
            .get_uniform_by_name("transform")
            .unwrap()
            .location
            .clone();
        let sampler = program
            .get_uniform_by_name("font_sampler")
            .unwrap()
            .location
            .clone();

        gl.use_shader(Some(&program));
        gl.set_uniform_by_location(&transform, &RawUniformValue::Mat4(IDENTITY_MATRIX.into()));
        gl.set_uniform_by_location(&sampler, &RawUniformValue::SignedInt(0));

        Pipeline {
            program,
            cache,
            instances,
            transform,
            current_transform: IDENTITY_MATRIX,
        }
    }

    pub fn draw(
        &mut self,
        gl: &mut solstice::Context,
        transform: [f32; 16],
        region: Option<Region>,
    ) {
        gl.use_shader(Some(&self.program));

        if self.current_transform != transform {
            gl.set_uniform_by_location(&self.transform, &RawUniformValue::Mat4(transform.into()));
            self.current_transform = transform;
        }

        let t = &self.cache.texture;
        gl.bind_texture_to_unit(t.get_texture_type(), t.get_texture_key(), 0.into());
        let geometry = self.instances.unmap(gl);
        solstice::Renderer::draw(
            gl,
            &self.program,
            &geometry,
            PipelineSettings {
                depth_state: None,
                scissor_state: region.map(|region| {
                    let Region {
                        x,
                        y,
                        width,
                        height,
                    } = region;
                    solstice::viewport::Viewport::new(x as _, y as _, width as _, height as _)
                }),
                ..Default::default()
            },
        );
    }

    pub fn update_cache(
        &mut self,
        gl: &mut solstice::Context,
        offset: [u16; 2],
        size: [u16; 2],
        data: &[u8],
    ) {
        unsafe {
            self.cache.update(gl, offset, size, data);
        }
    }

    pub fn increase_cache_size(&mut self, gl: &mut solstice::Context, width: u32, height: u32) {
        unsafe {
            self.cache.destroy(gl);

            self.cache = Cache::new(gl, width, height);
        }
    }

    pub fn upload(&mut self, quads: Vec<Quad<super::Vertex2D>>) {
        self.instances.clear();
        for quad in quads {
            let _id = self.instances.push(quad);
        }
    }
}

// Helpers
#[cfg_attr(rustfmt, rustfmt_skip)]
const IDENTITY_MATRIX: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];

#[derive(Debug, Clone, Copy, Vertex, Default)]
#[repr(C)]
pub struct Vertex2D {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

unsafe impl bytemuck::Zeroable for Vertex2D {}
unsafe impl bytemuck::Pod for Vertex2D {}

impl Vertex2D {
    const INITIAL_AMOUNT: usize = 50_000;

    pub fn from_vertex(
        glyph_brush::GlyphVertex {
            mut tex_coords,
            pixel_coords,
            bounds,
            extra,
        }: glyph_brush::GlyphVertex,
    ) -> Quad<Vertex2D> {
        let mut gl_rect = glyph_brush::ab_glyph::Rect {
            min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
            max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
        };

        // handle overlapping bounds, modify uv_rect to preserve texture aspect
        if gl_rect.max.x > bounds.max.x {
            let old_width = gl_rect.width();
            gl_rect.max.x = bounds.max.x;
            tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
        }
        if gl_rect.min.x < bounds.min.x {
            let old_width = gl_rect.width();
            gl_rect.min.x = bounds.min.x;
            tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
        }
        if gl_rect.max.y > bounds.max.y {
            let old_height = gl_rect.height();
            gl_rect.max.y = bounds.max.y;
            tex_coords.max.y =
                tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
        }
        if gl_rect.min.y < bounds.min.y {
            let old_height = gl_rect.height();
            gl_rect.min.y = bounds.min.y;
            tex_coords.min.y =
                tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
        }

        Quad {
            vertices: [
                Vertex2D {
                    position: [gl_rect.min.x as f32, gl_rect.min.y as f32],
                    uv: [tex_coords.min.x, tex_coords.min.y],
                    color: extra.color,
                },
                Vertex2D {
                    position: [gl_rect.max.x as f32, gl_rect.min.y as f32],
                    uv: [tex_coords.max.x, tex_coords.min.y],
                    color: extra.color,
                },
                Vertex2D {
                    position: [gl_rect.min.x as f32, gl_rect.max.y as f32],
                    uv: [tex_coords.min.x, tex_coords.max.y],
                    color: extra.color,
                },
                Vertex2D {
                    position: [gl_rect.max.x as f32, gl_rect.max.y as f32],
                    uv: [tex_coords.max.x, tex_coords.max.y],
                    color: extra.color,
                },
            ],
        }
    }
}

// unsafe fn create_program(
//     gl: &glow::Context,
//     shader_sources: &[(u32, &str)],
// ) -> <glow::Context as HasContext>::Program {
//     let program = gl.create_program().expect("Cannot create program");
//
//     let mut shaders = Vec::with_capacity(shader_sources.len());
//
//     for (shader_type, shader_source) in shader_sources.iter() {
//         let shader = gl
//             .create_shader(*shader_type)
//             .expect("Cannot create shader");
//
//         gl.shader_source(shader, shader_source);
//         gl.compile_shader(shader);
//
//         if !gl.get_shader_compile_status(shader) {
//             panic!(gl.get_shader_info_log(shader));
//         }
//
//         gl.attach_shader(program, shader);
//
//         shaders.push(shader);
//     }
//
//     gl.link_program(program);
//     if !gl.get_program_link_status(program) {
//         panic!(gl.get_program_info_log(program));
//     }
//
//     for shader in shaders {
//         gl.detach_shader(program, shader);
//         gl.delete_shader(shader);
//     }
//
//     program
// }

// unsafe fn create_instance_buffer(
//     gl: &glow::Context,
//     size: usize,
// ) -> (
//     <glow::Context as HasContext>::VertexArray,
//     <glow::Context as HasContext>::Buffer,
// ) {
//     let vertex_array = gl.create_vertex_array().expect("Create vertex array");
//     let buffer = gl.create_buffer().expect("Create instance buffer");
//
//     gl.bind_vertex_array(Some(vertex_array));
//     gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
//     gl.buffer_data_size(
//         glow::ARRAY_BUFFER,
//         (size * std::mem::size_of::<Instance>()) as i32,
//         glow::DYNAMIC_DRAW,
//     );
//
//     let stride = std::mem::size_of::<Instance>() as i32;
//
//     gl.enable_vertex_attrib_array(0);
//     gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
//     gl.vertex_attrib_divisor(0, 1);
//
//     gl.enable_vertex_attrib_array(1);
//     gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 4 * 3);
//     gl.vertex_attrib_divisor(1, 1);
//
//     gl.enable_vertex_attrib_array(2);
//     gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, stride, 4 * (3 + 2));
//     gl.vertex_attrib_divisor(2, 1);
//
//     gl.enable_vertex_attrib_array(3);
//     gl.vertex_attrib_pointer_f32(3, 2, glow::FLOAT, false, stride, 4 * (3 + 2 + 2));
//     gl.vertex_attrib_divisor(3, 1);
//
//     gl.enable_vertex_attrib_array(4);
//     gl.vertex_attrib_pointer_f32(4, 4, glow::FLOAT, false, stride, 4 * (3 + 2 + 2 + 2));
//     gl.vertex_attrib_divisor(4, 1);
//
//     gl.bind_vertex_array(None);
//     gl.bind_buffer(glow::ARRAY_BUFFER, None);
//
//     (vertex_array, buffer)
// }
