use std::{
    env, fs, iter,
    path::{Component, Path, PathBuf},
};

use shaderc::{Compiler, ShaderKind};
use walkdir::WalkDir;

/// Use this in your `build.rs` to compile all of your shaders along with the rest of your code.
///
/// `CompilationRun` looks for the following file extensions and interprets them as specified below:
/// ```no_test
/// ".vert" => Vertex Shader
/// ".vs" => Vertex Shader
/// ".frag" => Fragment Shader
/// ".fs" => Fragment Shader
/// ".gs" => Geometry Shader
/// ".geom" => Geometry Shader
/// ".comp" => Compute Shader
/// ".tesc" => Tesselation Control Shader
/// ".tese" => Tesselation Evaluation Shader
/// ".rgen" => Ray Generation Shader
/// ".rint" => Ray Intersecion Shader
/// ".rahit" => Ray Any Hit Shader
/// ".rchit" => Ray Closest Shader
/// ".rmiss" => Ray Miss Shader
/// ".rcall" => Ray Callable Shader
/// ".mesh" => Mesh Shader
/// ".task" => Task Shader
/// ".glsl" => Guess from contents
/// ```
pub struct CompilationRun<'a> {
    directories: Vec<&'a Path>,
    max_depth: usize,
}

impl<'a> CompilationRun<'a> {
    pub fn new(dir: &'a Path) -> Self {
        Self {
            directories: vec![dir],
            max_depth: 0,
        }
    }

    /// Add a directory which will be searched for shaders.
    pub fn with_dir(mut self, dir: &'a Path) -> Self {
        self.directories.push(dir);
        self
    }

    /// How many directores deep should we look for shaders before we give up?
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Look for shaders and compile them.
    pub fn run(self) {
        let mut compiler = Compiler::new().expect("Could not initialize shader compiler");

        let mut errors = Vec::new();

        for dir in self.directories {
            for entry in WalkDir::new(dir).min_depth(1).max_depth(self.max_depth) {
                let entry = entry.unwrap();
                if entry.path().is_dir() {
                    continue;
                }

                if let Some(file_ext) = entry.path().extension().map(|ext| ext.to_string_lossy()) {
                    let shader_kind = match file_ext.as_ref() {
                        "vert" => Some(ShaderKind::Vertex),
                        "vs" => Some(ShaderKind::Vertex),
                        "frag" => Some(ShaderKind::Fragment),
                        "fs" => Some(ShaderKind::Fragment),
                        "gs" => Some(ShaderKind::Geometry),
                        "geom" => Some(ShaderKind::Geometry),
                        "comp" => Some(ShaderKind::Compute),
                        "tesc" => Some(ShaderKind::TessControl),
                        "tese" => Some(ShaderKind::TessEvaluation),
                        "rgen" => Some(ShaderKind::RayGeneration),
                        "rint" => Some(ShaderKind::Intersection),
                        "rahit" => Some(ShaderKind::AnyHit),
                        "rchit" => Some(ShaderKind::ClosestHit),
                        "rmiss" => Some(ShaderKind::Miss),
                        "rcall" => Some(ShaderKind::Callable),
                        "mesh" => Some(ShaderKind::Mesh),
                        "task" => Some(ShaderKind::Task),
                        "glsl" => Some(ShaderKind::InferFromSource),
                        _ => None,
                    };

                    if let Some(shader_kind) = shader_kind {
                        let source_text = match fs::read_to_string(entry.path()) {
                            Ok(ok) => ok,
                            Err(err) => {
                                errors.push(err.into());
                                continue;
                            }
                        };
                        match compiler.compile_into_spirv(
                            &source_text,
                            shader_kind,
                            &entry.path().display().to_string(),
                            "main",
                            None,
                        ) {
                            Ok(artifact) => {
                                let spirv_dir =
                                    PathBuf::from(env::var("OUT_DIR").unwrap()).join("SPIR-V");
                                match fs::create_dir(&spirv_dir) {
                                    Ok(ok) => ok,
                                    Err(err) => match err.kind() {
                                        std::io::ErrorKind::AlreadyExists => {}
                                        _ => {
                                            errors.push(err.into());
                                            continue;
                                        }
                                    },
                                }
                                let artifact_name = shader_path_to_file_name(entry.path());
                                match fs::write(
                                    spirv_dir.join(artifact_name),
                                    artifact.as_binary_u8(),
                                ) {
                                    Ok(ok) => ok,
                                    Err(err) => {
                                        errors.push(err.into());
                                        continue;
                                    }
                                };
                            }
                            Err(err) => errors.push(
                                ShaderCompileFail {
                                    path: entry.into_path(),
                                    error: err,
                                }
                                .into(),
                            ),
                        }
                    }
                }
            }
        }

        if !errors.is_empty() {
            for error in errors.iter() {
                match error {
                    CompilationRunError::Io(err) => {
                        eprintln!("IO error: {}", err);
                    }
                    CompilationRunError::CompileFail(err) => {
                        eprintln!(
                            r#"Error compiling shader at "{}": {}"#,
                            err.path.display(),
                            err.error,
                        );
                    }
                }
            }

            panic!(
                "{} errors were encountered while attempting to compile shaders.",
                errors.len()
            );
        }
    }
}

enum CompilationRunError {
    Io(std::io::Error),
    CompileFail(ShaderCompileFail),
}

impl From<std::io::Error> for CompilationRunError {
    fn from(from: std::io::Error) -> Self {
        Self::Io(from)
    }
}

impl From<ShaderCompileFail> for CompilationRunError {
    fn from(from: ShaderCompileFail) -> Self {
        Self::CompileFail(from)
    }
}

struct ShaderCompileFail {
    path: PathBuf,
    error: shaderc::Error,
}

fn shader_path_to_file_name<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name),
            _ => None,
        })
        .map(|name| name.to_string_lossy())
        .zip(
            iter::successors(Some(0), |prev| match prev {
                _ => Some(1),
            })
            .map(|state| match state {
                0 => "",
                _ => "__",
            }),
        )
        .map(|(name, extra)| iter::once(extra.into()).chain(iter::once(name)))
        .flatten()
        .chain(iter::once(".spirv".into()))
        .collect()
}
