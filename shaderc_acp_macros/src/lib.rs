#[doc(hidden)]
pub use bytemuck;

// TODO: Make this a procedural macro which takes the current relative path to the shader as its input
//       Blocked on [rust-lang/rust#54725](https://github.com/rust-lang/rust/issues/54725)
#[macro_export]
macro_rules! include_shader {
    ($shader:literal) => {{
        let shader_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/SPIR-V/", $shader, ".spirv"));
        $crate::bytemuck::cast_slice(shader_bytes)
    }};
}
