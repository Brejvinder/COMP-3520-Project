use std::ffi::{ CString, CStr };
use std::fs;
use std::path::Path;
use std::ptr;

/// Loads a OpenGL shader program and cleans up when it's deleted.
#[derive(Debug)]
pub struct ShaderProgram(gl::types::GLuint);

impl ShaderProgram {
    pub fn load_from<V, F>(v: V, f: F) -> Result<Self, String>
        where V: AsRef<Path>,
              F: AsRef<Path>
    {
        let vertex_shader = Shader::load_from(v, gl::VERTEX_SHADER)?;
        let fragment_shader = Shader::load_from(f, gl::FRAGMENT_SHADER)?;
        
        unsafe {
            let id = gl::CreateProgram();
            
            gl::AttachShader(id, vertex_shader.0);
            gl::AttachShader(id, fragment_shader.0);
            gl::LinkProgram(id);

            let mut len = 0;
            gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            
            let res = if len > 0 {
                let mut buf = vec![0; len as usize];
                gl::GetProgramInfoLog(id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);

                let s = CStr::from_bytes_with_nul(&buf)
                    .expect("Shader info log has malformed nul")
                    .to_string_lossy()
                    .to_string();

                gl::DeleteProgram(id);

                Err(s)
            } else {
                Ok(ShaderProgram(id))
            };

            gl::DetachShader(id, vertex_shader.0);
            gl::DetachShader(id, fragment_shader.0);

            res
        }

    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.0);
        }
    }
}

/// Helper type to create, compile, and delete shaders easily.
#[derive(Debug)]
struct Shader(gl::types::GLuint);

impl Shader {
    fn load_from<P>(p: P, ty: gl::types::GLenum) -> Result<Self, String>
        where P: AsRef<Path>
    {
        let path = p.as_ref();
        
        let source = fs::read_to_string(path)
            .map_err(|_| format!("Could not load shader from {:?}", path))
            .map(|s| CString::new(s).unwrap())?;

        unsafe {
            let id = gl::CreateShader(ty);
            
            gl::ShaderSource(id, 1, [source.as_ptr()].as_ptr() as *const _ as *const *const _, ptr::null_mut());
            gl::CompileShader(id);

            let mut len = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);

            if len > 0 {
                let mut buf = vec![0; len as usize];
                gl::GetShaderInfoLog(id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut _);

                let s = CStr::from_bytes_with_nul(&buf)
                    .expect("Shader info log has malformed nul")
                    .to_string_lossy()
                    .to_string();

                gl::DeleteShader(id);

                Err(s)
            } else {
                Ok(Shader(id))
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}