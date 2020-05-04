// TODO: Make this a procedural macro which takes the current relative path to the shader as its input
//       Blocked on [rust-lang/rust#54725](https://github.com/rust-lang/rust/issues/54725)
#[macro_export]
macro_rules! include_shader {
    ($shader:literal) => {{
        let shader_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/SPIR-V/", $shader, ".spirv"));
        if shader_bytes.len() % std::mem::size_of::<u32>() != 0 {
            panic!(concat!(
                "Shader ",
                $shader,
                " contains a number of bytes which is not divisible by four."
            ));
        }
        $crate::bytemuck::cast_slice(shader_bytes)
    }};
}
