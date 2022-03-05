use image::{imageops::FilterType, RgbImage, RgbaImage};

use wgpu::{include_wgsl, util::DeviceExt, BindGroup, BindGroupLayout, Device, Queue};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const TARGET: &str = "planet.jpeg";

fn main() {
    let width = 500;
    let img = image::open(TARGET).unwrap();
    let aspect_ratio = img.width() as f32 / img.height() as f32;
    let height: u32 = (width as f32 * aspect_ratio) as u32;

    let target = img.resize(width, height, FilterType::Triangle);

    // spritesheets
    let sheet1data: serde_json::Value =
        serde_json::from_str(include_str!("../spritesheets/sheet1.json")).unwrap();
    let sheet2data: serde_json::Value =
        serde_json::from_str(include_str!("../spritesheets/sheet2.json")).unwrap();

    env_logger::init();
    let event_loop = EventLoop::new();
    //let window = WindowBuilder::new().build(&event_loop).unwrap();
    // create window with size 400 x 400
    // that cannot be resized
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(
            target.width(),
            target.height(),
        ))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();
    // State::new uses async code, so we're going to wait for it to finish
    let mut state = pollster::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
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
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                //Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}

use winit::window::Window;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    obj_textures: Vec<texture::Texture>,
    bind_group: BindGroup,
    vertex_buffer: wgpu::Buffer,
    // size_buffer: wgpu::Buffer,
    size_bind_group: BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [u32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Uint32x2, 1 => Float32x2];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SizeUniform {
    width: u32,
    height: u32,
}

// square with our texture
const VERTICES: &[Vertex] = &[
    // triangle 1
    Vertex {
        position: [0, 50],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0, 0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [50, 0],
        tex_coords: [1.0, 1.0],
    },
    // triangle 2
    Vertex {
        position: [0, 50],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [50, 0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [50, 50],
        tex_coords: [1.0, 0.0],
    },
];

mod texture;

impl State {
    // Creating some of the wgpu types requires async code

    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let img = include_bytes!("../spritesheets/sheet2.png");

        let sheet_texture =
            texture::Texture::from_bytes(&device, &queue, img, "sheet2.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let sheet_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sheet_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sheet_texture.sampler),
                },
            ],
            label: Some("sheet_bind_group"),
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let size_uniform = SizeUniform {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };

        let size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Size buffer"),
            contents: bytemuck::cast_slice(&[size_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let size_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("size_bind_group_layout"),
            });

        let size_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &size_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: size_buffer.as_entire_binding(),
            }],
            label: Some("size_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &size_bind_group_layout],
                push_constant_ranges: &[],
            });

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
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
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

        Self {
            surface,
            device,
            queue,

            render_pipeline,
            vertex_buffer,
            obj_textures: vec![sheet_texture],
            bind_group: sheet_bind_group,
            //size_buffer,
            size_bind_group,
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_bind_group(1, &self.size_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..VERTICES.len() as u32, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

use rand::Rng;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy)]
struct Shape {
    img_index: usize,
    x: i32,
    y: i32,
    scale: f32,
    rot: f32,
}

const OBJ_IDS: &[u16] = &[
    //211, 259, 266, 273, 280, 693, 695, 697, 699, 701, 725,
    1011, 1012, 1013, 1102, 1106, 1111, 1112, 1113, 1114, 1115, 1116, 1117, 1118, 1348, 1351, 1352,
    1353, 1354, 1355, 1442, 1443, 1461, 1462, 1463, 1464, 1596, 1597, 1608, 1609, 1610, 1753, 1754,
    1757, 1764, 1765, 1766, 1767, 1768, 1769, 1837, 1835, 1869, 1870, 1871, 1874, 1875, 1886, 1887,
    1888,
];

impl Shape {
    fn paste(&self, img: &mut RgbImage, device: &wgpu::Device, spritesheet: serde_json::Value) {
        let sprite = &spritesheet["frames"][format!("{}_main.png", OBJ_IDS[self.img_index])];
        let sprite_x = sprite["frame"]["x"].as_u64().unwrap() as u32;
        let sprite_y = sprite["frame"]["y"].as_u64().unwrap() as u32;
        let sprite_w = sprite["frame"]["w"].as_u64().unwrap() as u32;
        let sprite_h = sprite["frame"]["h"].as_u64().unwrap() as u32;
    }

    fn new_random(width: u32, height: u32, img_index: usize) -> Shape {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..width) as i32;
        let y = rng.gen_range(0..height) as i32;
        let scale = rng.gen_range(0.2..3.0);
        let rot = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));

        Shape {
            img_index,
            x,
            y,
            scale,
            rot,
        }
    }

    fn adjust_random(&mut self) {
        self.x = self.x as i32 + rand::thread_rng().gen_range(-3i32..=3);
        self.y = self.y as i32 + rand::thread_rng().gen_range(-3i32..=3);
        self.scale *= rand::thread_rng().gen_range(0.9..1.1);
        self.rot += rand::thread_rng().gen_range(-0.1..0.1);
    }
}

fn rotate_point(x: f32, y: f32, angle: f32) -> (f32, f32) {
    let cos = angle.cos();
    let sin = angle.sin();
    (x * cos - y * sin, x * sin + y * cos)
}
