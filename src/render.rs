use crate::shader;

#[derive(Clone, Copy, Default, glsl_layout::AsStd140)]
struct ShaderUniforms {
    width: u32,
    height: u32,
}

unsafe impl bytemuck::Pod for ShaderUniforms {}
unsafe impl bytemuck::Zeroable for ShaderUniforms {}

pub struct Renderer {
    uniform_buffer: wgpu::Buffer,
    storage_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new(device: &mut wgpu::Device, width: u32, height: u32) -> Renderer {
        let vs_module = shader::load("shaders/quad.vert", &device);
        let fs_module = shader::load("shaders/render.frag", &device);
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<ShaderUniforms>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<[f32; 4]>() * width as usize * height as usize) as u64,
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
            ],
            label: None,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bindgroup_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8Unorm,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bindgroup_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(storage_buffer.slice(..)),
                },
            ],
        });

        Renderer {
            uniform_buffer,
            storage_buffer,
            render_pipeline,
            bind_group,
            width,
            height,
        }
    }

    pub fn render<'r>(
        &'r mut self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'r>,
        img: &Vec<f32>,
    ) {
        let shader_uniforms = ShaderUniforms {
            width: self.width,
            height: self.height,
        };
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&shader_uniforms),
        );
        queue.write_buffer(&self.storage_buffer, 0, bytemuck::cast_slice(&img[..]));
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
