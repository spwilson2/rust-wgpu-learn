fn main() {
    pollster::block_on(run());
}

use image::EncodableLayout;
use std::{default, iter, mem::size_of};
use wgpu::{
    util::DeviceExt, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BufferBindingType, Limits, RenderPassDepthStencilAttachment, ShaderStages, TextureUsages,
    TextureViewDescriptor,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        color: [1.0, 1.0, 1.0],
    },
];
#[rustfmt::skip]
const INDICES: &[u16] = &[
    3, 2, 1,
    3, 0, 1,
];

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformExample {
    color: [f32; 4],
    time: f32,
    _pad: [f32; 3],
}
//const UNIFORM: &[UniformExample] = &[UniformExample { utime: 0.0 }];

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: Option<wgpu::Texture>,
    depth_texture_view: Option<wgpu::TextureView>,
    display_texture: wgpu::Texture,
    display_texture_view: wgpu::TextureView,
    texture_depth_format: wgpu::TextureFormat,
    timestamp: std::time::Instant,
    num_indices: u32,
    window: Window,
    count: usize,
}

fn gen_texture_data2(width: usize, height: usize) -> Vec<[u8; 4]> {
    let mut res_arr = vec![];
    for i in 0..width {
        for j in 0..height {
            res_arr.push([0 as u8, 0 as u8, 0, 255]);
        }
    }
    let res = std::fs::read("assets/sshot.png").unwrap();
    let res = image::load_from_memory_with_format(res.as_slice(), image::ImageFormat::Png);
    let res = res.unwrap().into_rgba8();
    assert!(res.width() < width as u32);
    assert!(res.height() < height as u32);
    let res_bytes = res.as_bytes();
    for i in 0..res.width() {
        for j in 0..res.height() {
            res_arr[(i + j * width as u32) as usize][0] = res_bytes[(i + j * res.width()) as usize];
            res_arr[(i + j * width as u32) as usize][1] =
                res_bytes[(i + j * res.width()) as usize + 1];
            res_arr[(i + j * width as u32) as usize][2] =
                res_bytes[(i + j * res.width()) as usize + 2];
            res_arr[(i + j * width as u32) as usize][3] =
                res_bytes[(i + j * res.width()) as usize + 3];
        }
    }
    res_arr
}
fn gen_texture_data(width: usize, height: usize) -> (Vec<u8>, usize, usize) {
    let res = std::fs::read("assets/sshot.png").unwrap();
    let res = image::load_from_memory_with_format(res.as_slice(), image::ImageFormat::Png);
    let res = res.unwrap().into_rgba8();
    (
        res.iter().copied().collect(),
        res.width() as usize,
        res.height() as usize,
    )
}

impl State {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    limits: Limits {
                        //max_bind_groups: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                /*trace_path=*/ None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Example bind group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // The binding index as used in the @binding attribute in the shader
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            size_of::<UniformExample>() as _
                        ),
                    },
                    //count: std::num::NonZeroU32::new(1),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // The binding index as used in the @binding attribute in the shader
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_layout],
                push_constant_ranges: &[],
            });

        let texture_depth_format = wgpu::TextureFormat::Depth24Plus;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                //cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                //polygon_mode: wgpu::PolygonMode::Line,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture_depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<UniformExample>() as u64,
            mapped_at_creation: false,
        });

        let display_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("z-Depth texture"),
            size: wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let display_texture_view = display_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Display texture view"),
            format: Some(display_texture.format()),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: Default::default(),
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: std::num::NonZeroU64::new(size_of::<UniformExample>() as _),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&display_texture_view),
                },
            ],
        });

        let mut s = Self {
            count: 0,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            texture_depth_format,
            depth_texture: None,
            depth_texture_view: None,
            display_texture,
            display_texture_view,
            num_indices,
            window,
            timestamp: std::time::Instant::now(),
        };
        s.configue_texture_depth_buffer();
        s
    }

    fn configue_texture_depth_buffer(&mut self) {
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("z-Depth texture"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_depth_format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[self.texture_depth_format],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("z-Depth texture view"),
            format: Some(self.texture_depth_format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });

        self.depth_texture = Some(depth_texture);
        self.depth_texture_view = Some(depth_texture_view);
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.configue_texture_depth_buffer();
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.count += 1;
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: self.depth_texture_view.as_ref().unwrap(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(depth_stencil_attachment),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            //render_pass.draw(0..VERTICES.len() as u32, 0..1);
        }
        let (tex, width, height) =
            gen_texture_data(self.config.width as usize, self.config.height as usize);

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.display_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            bytemuck::cast_slice(tex.as_slice()),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width as u32),
                rows_per_image: Some(height as u32),
            },
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            //wgpu::Extent3d {
            //    width: self.display_texture.width(),
            //    height: self.display_texture.height(),
            //    depth_or_array_layers: 1,
            //},
        );

        self.queue.write_buffer(
            &self.uniform_buffer,
            /*offset=*/ 0,
            bytemuck::bytes_of(&UniformExample {
                color: [0.5, 1.0, 0.4, 1.0],
                time: self.timestamp.elapsed().as_secs_f32(),
                ..Default::default()
            }),
        );
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;
    let mut timer = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                //println!("Time: {}", timer.elapsed().as_millis());
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
