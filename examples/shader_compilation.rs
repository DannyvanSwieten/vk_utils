use shaderc::ShaderKind;
use vk_utils::shader_compiler::ShaderCompiler;

pub fn main() {
    let src = r"
    #version 450
    layout(set = 0, binding = 0) uniform sampler2D tex[1024];
    layout(location = 0) out vec4 position;
    void main(){}
    ";
    let result = ShaderCompiler::compile_string(src, ShaderKind::Vertex, "", "main");
    if !result.failed() {
        let reflection = result.reflect();
        if let Some(descriptor_sets) = reflection.descriptor_sets() {
            for (set, descriptors) in &descriptor_sets {
                for (binding, descriptor) in descriptors {
                    println!("Set: {}, Binding: {},  {:?}", set, binding, descriptor);
                }
            }
        }
    } else {
        println!("Error: {}", result.error_string())
    }
}
