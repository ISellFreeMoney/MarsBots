//! Helpers for pipeline creation and initialization
use std::path::Path;
use wgpu::{FragmentState, VertexState};
use wgpu_types::{BlendComponent, BlendFactor, BlendOperation, BlendState};

/// Shader stage
pub enum ShaderStage {
    Vertex,
    Fragment,
}

/// Load a GLSL shader from a file and compile it to SPIR-V
pub fn load_glsl_shader<'a, P: AsRef<Path>>(stage: ShaderStage, path: P) -> Vec<u8> {
    let ty = match stage {
        ShaderStage::Vertex => shaderc::ShaderKind::Vertex,
        ShaderStage::Fragment => shaderc::ShaderKind::Fragment,
    };
    let path_display = path.as_ref().display().to_string();
    log::info!("Loading GLSL shader from {}", path_display);
    let glsl_source = std::fs::read_to_string(path).expect("Couldn't read shader from file");

    let mut compiler = shaderc::Compiler::new().unwrap();
    compiler.compile_into_spirv(&glsl_source, ty, &path_display, "main", None)
        .expect("Couldn't compile shader.")
        .as_binary_u8()
        .to_vec()
}

/// Default `RasterizationStateDescriptor` with no backface culling
//pub const RASTERIZER_NO_CULLING: wgpu::RasterizationStateDescriptor =
    //wgpu::RasterizationStateDescriptor {
    //    front_face: wgpu::FrontFace::Ccw,
    //    cull_mode: wgpu::CullMode::None,
  //      depth_bias: 0,
//        depth_bias_slope_scale: 0.0,
//        depth_bias_clamp: 0.0,
//        clamp_depth: false
//    };

/// Default `RasterizationStateDescriptor` with backface culling
//pub const RASTERIZER_WITH_CULLING: wgpu::Rast =
 //   wgpu::RasterizationStateDescriptor {
  //      cull_mode: wgpu::CullMode::Back,
   //     ..RASTERIZER_NO_CULLING
    //};

/// Default `ColorStateDescriptor`
pub const DEFAULT_COLOR_STATE_DESCRIPTOR: [wgpu::ColorTargetState; 1] =
    [wgpu::ColorTargetState {
        format: crate::window::COLOR_FORMAT,

        write_mask: wgpu::ColorWrites::ALL,
        blend: Option::from(BlendState {
            alpha: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
        }, color: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            }
        }),
    }];

/// Default `DepthStencilStateDescriptor`
pub const DEFAULT_DEPTH_STENCIL_STATE_DESCRIPTOR: wgpu::DepthStencilState =
    wgpu::DepthStencilState {
        format: crate::window::DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState {
            front: wgpu::StencilFaceState::IGNORE,
            back: wgpu::StencilFaceState::IGNORE,
            read_mask: 0,
            write_mask: 0,
        },
        bias: Default::default(),
    };

/// Create a default pipeline
pub fn create_default_pipeline(
    device: &wgpu::Device,
    uniform_layout: &wgpu::BindGroupLayout,
    vertex_shader: wgpu::ShaderModuleDescriptor,
    fragment_shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    // Shaders
    let vertex_shader_module = device.create_shader_module(vertex_shader);
    let fragment_shader_module = device.create_shader_module(fragment_shader);

    // Pipeline
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[uniform_layout],
        push_constant_ranges: &[]
    });

    log::trace!("Creating render pipeline.");

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &vertex_shader_module,
            entry_point: "main",
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: Some(DEFAULT_DEPTH_STENCIL_STATE_DESCRIPTOR),
        multisample: Default::default(),
        fragment: Option::from(FragmentState {
            module: &fragment_shader_module,
            entry_point: "main",
            compilation_options: Default::default(),
            targets: &[],
        }),
        multiview: None,
        cache: None,
    })
}
