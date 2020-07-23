use std::error::Error;

use shaderc::{CompilationArtifact, Compiler, ShaderKind};

pub fn load(path: &str, device: &wgpu::Device) -> wgpu::ShaderModule {
    let spirv = match compile_glsl(path) {
        Ok(spirv) => spirv,
        Err(e) => panic!("Failed to load shader {}\n{}", path, e),
    };
    device.create_shader_module(wgpu::ShaderModuleSource::SpirV(spirv.as_binary().into()))
}

fn compile_glsl(path: &str) -> Result<CompilationArtifact, Box<dyn Error>> {
    let mut compiler = Compiler::new().unwrap();
    let _path = std::path::Path::new(path);
    let extension = _path.extension().unwrap().to_str().unwrap();
    let shader_kind = match extension {
        "vert" => ShaderKind::Vertex,
        "frag" => ShaderKind::Fragment,
        "comp" => ShaderKind::Compute,
        _ => return Err(format!("Invalid shader extension {}", extension).into()),
    };
    let source = std::fs::read_to_string(path)?;
    let spirv = compiler.compile_into_spirv(&source, shader_kind, path, "main", None)?;
    Ok(spirv)
}
