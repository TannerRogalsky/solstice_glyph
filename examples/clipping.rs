use solstice::ClearSettings;
use solstice_glyph::{ab_glyph, GlyphBrushBuilder, Region, Section, Text};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // Open window and create a surface
    let event_loop = glutin::event_loop::EventLoop::new();

    let window_builder = glutin::window::WindowBuilder::new().with_resizable(false);

    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)
        .expect("Open window");

    let context = unsafe { context.make_current().expect("Make OpenGL context current") };

    let mut size = context.window().inner_size();

    // Initialize OpenGL
    let gl = unsafe {
        solstice::glow::Context::from_loader_function(|s| context.get_proc_address(s) as *const _)
    };
    let mut gl = solstice::Context::new(gl);

    // Prepare glyph_brush
    let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!("Inconsolata-Regular.ttf"))?;

    let mut glyph_brush = GlyphBrushBuilder::using_font(inconsolata).build(&mut gl);

    // Render loop
    context.window().request_redraw();

    event_loop.run(move |event, _, control_flow| match event {
        glutin::event::Event::WindowEvent {
            event: glutin::event::WindowEvent::CloseRequested,
            ..
        } => *control_flow = glutin::event_loop::ControlFlow::Exit,
        glutin::event::Event::WindowEvent {
            event: glutin::event::WindowEvent::Resized(new_size),
            ..
        } => {
            context.resize(new_size);
            gl.set_viewport(0, 0, new_size.width as _, new_size.height as _);

            size = new_size;
        }
        glutin::event::Event::RedrawRequested { .. } => {
            let clear_color = solstice::Color {
                red: 0.4,
                blue: 0.4,
                green: 0.4,
                alpha: 1.,
            };
            solstice::Renderer::clear(
                &mut gl,
                ClearSettings {
                    color: Some(clear_color.into()),
                    ..Default::default()
                },
            );

            glyph_brush.queue(Section {
                screen_position: (30.0, 30.0),
                bounds: (size.width as f32, size.height as f32),
                text: vec![Text::default()
                    .with_text("Hello glow_glyph!")
                    .with_color([0.0, 0.0, 0.0, 1.0])
                    .with_scale(40.0)],
                ..Section::default()
            });

            glyph_brush
                .draw_queued(&mut gl, size.width, size.height)
                .expect("Draw queued");

            glyph_brush.queue(Section {
                screen_position: (30.0, 90.0),
                bounds: (size.width as f32, size.height as f32),
                text: vec![Text::default()
                    .with_text("Hello glow_glyph!")
                    .with_color([1.0, 1.0, 1.0, 1.0])
                    .with_scale(40.0)],
                ..Section::default()
            });

            glyph_brush
                .draw_queued_with_transform_and_scissoring(
                    &mut gl,
                    solstice_glyph::orthographic_projection(size.width, size.height),
                    Region {
                        x: 40,
                        y: size.height - 120,
                        width: 200,
                        height: 15,
                    },
                )
                .expect("Draw queued");

            context.swap_buffers().expect("Swap buffers");
        }
        _ => {
            *control_flow = glutin::event_loop::ControlFlow::Wait;
        }
    })
}
