use clipboard::{ClipboardContext, ClipboardProvider};
use imgui::*;
use imgui_wgpu::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use crate::window::WindowContext;

struct Clipboard(ClipboardContext);
impl ClipboardBackend for Clipboard {
    fn get(&mut self) -> Option<ImString> {
        self.0.get_contents().ok().map(|text| text.into())
    }
    fn set(&mut self, text: &ImStr) {
        let _ = self.0.set_contents(text.to_str().to_owned());
    }
}

pub struct ImguiContext {
    pub context: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
}

impl ImguiContext {
    pub fn new(window_context: &mut WindowContext) -> ImguiContext {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        if let Ok(clipboard) = ClipboardContext::new() {
            imgui.set_clipboard_backend(Box::new(Clipboard(clipboard)));
        } else {
            eprintln!("Failed to initialize clipboard");
        }

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window_context.window, HiDpiMode::Default);

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        let renderer = Renderer::new(
            &mut imgui,
            &window_context.device,
            &mut window_context.queue,
            wgpu::TextureFormat::Bgra8Unorm,
        );

        ImguiContext {
            context: imgui,
            platform,
            renderer,
            font_size,
        }
    }

    pub fn render<'r>(
        &'r mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        window: &winit::window::Window,
        render_pass: &mut wgpu::RenderPass<'r>,
        delta_time: &std::time::Duration,
    ) {
        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("Failed to prepare frame");
        let ui = self.context.frame();
        compose(&ui, delta_time);
        self.platform.prepare_render(&ui, &window);
        self.renderer
            .render(ui.render(), queue, device, render_pass)
            .expect("Rendering failed");
    }
}

fn compose(ui: &imgui::Ui, delta_time: &std::time::Duration) {
    let window = imgui::Window::new(im_str!("Fluid Simulation"));
    window
        .size([300.0, 100.0], Condition::FirstUseEver)
        .build(&ui, || {
            let mouse_pos = ui.io().mouse_pos;
            ui.text(im_str!(
                "Mouse Position: ({:.1},{:.1})",
                mouse_pos[0],
                mouse_pos[1]
            ));
            ui.text(im_str!("Frametime: {:?}", delta_time));
        });
}
