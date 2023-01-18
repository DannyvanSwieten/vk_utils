use shaderc::ShaderKind;
use vk_utils::shader_compiler::ShaderCompiler;

pub fn main() {
    let src = r"
    #version 450
    layout(location = 0) out vec4 position;
    void main(){}
    ";
    let result = ShaderCompiler::compile_string(src, ShaderKind::Vertex, "", "main");
    if !result.failed() {
        let reflection = result.reflect();
        if let Some(outputs) = reflection.outputs() {
            for output in &outputs {
                println!("Output: {}", output.name);
            }
        }
    } else {
        println!("Error: {}", result.error_string())
    }
}
