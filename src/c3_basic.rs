extern mod glfw3;
extern mod glcore;
extern mod stb_image;

use cast::transmute;
use ptr::{is_null, null, to_unsafe_ptr};
use str::as_c_str;
use sys::size_of;
use vec::raw::to_ptr;

use glcore::*;
use stb_image::image::load_with_depth;

fn main() {
    do task::task().sched_mode(task::PlatformThread).spawn {
        if (glfw3::init() == 0) {
            glfw3::terminate();
            fail(~"glfwInit() failed\n");
        }
        
        // Choose a GL profile that is compatible with OS X 10.7+
        glfw3::window_hint(glfw3::OPENGL_VERSION_MAJOR, 3);
        glfw3::window_hint(glfw3::OPENGL_VERSION_MINOR, 2);
        glfw3::window_hint(glfw3::OPENGL_PROFILE, glfw3::OPENGL_CORE_PROFILE);
        glfw3::window_hint(glfw3::OPENGL_FORWARD_COMPAT, 1);
        
        let mut window = glfw3::create_window(800, 600, glfw3::WINDOWED, ~"OpenGL");
        
        if (is_null(window.ptr)) {
            glfw3::terminate();
            io::println(~"Error: " + glfw3::error_string(glfw3::get_error()));
            fail(~"glfwOpenWindow() failed\n");
        }
        
        window.make_context_current();
        
        // Create Vertex Array Object
        let vao: GLuint = 0;
        glGenVertexArrays(1, to_unsafe_ptr(&vao));
        glBindVertexArray(vao);
        
        // Create a Vertex Buffer Object and copy the vertex data to it
        let vbo: GLuint = 0;
        glGenBuffers(1, to_unsafe_ptr(&vbo));
        
        let vertices: [GLfloat * 28] = [
        //   Position     Color            Texcoords
            -0.5,  0.5,   1.0, 0.0, 0.0,   0.0, 0.0, // Top-left
             0.5,  0.5,   0.0, 1.0, 0.0,   1.0, 0.0, // Top-right
             0.5, -0.5,   0.0, 0.0, 1.0,   1.0, 1.0, // Bottom-right
            -0.5, -0.5,   1.0, 1.0, 1.0,   0.0, 1.0  // Bottom-left
        ];
        
        glBindBuffer(GL_ARRAY_BUFFER, vbo);
        unsafe {
            glBufferData(GL_ARRAY_BUFFER,
                         (vertices.len() * size_of::<GLfloat>()) as GLsizeiptr,
                         transmute(to_ptr(vertices)),
                         GL_STATIC_DRAW);
        }
        
        // Create an element array
        let ebo: GLuint = 0;
        glGenBuffers(1, to_unsafe_ptr(&ebo));
        
        let elements: [GLuint * 6] = [
            0, 1, 2,
            2, 3, 0
        ];
        
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
        unsafe {
            glBufferData(GL_ELEMENT_ARRAY_BUFFER,
                         (elements.len() * size_of::<GLfloat>()) as GLsizeiptr,
                         transmute(to_ptr(elements)),
                         GL_STATIC_DRAW);
        }
        
        // Shader sources
        let vertexSource =
          ~"#version 150\n\
            in vec2 position;\n\
            in vec3 color;\n\
            in vec2 texcoord;\n\
            out vec3 Color;\n\
            out vec2 Texcoord;\n\
            void main() {\n\
                Color = color;\n\
                Texcoord = texcoord;\n\
                gl_Position = vec4(position, 0.0, 1.0);\n\
            }";

        let fragmentSource =
          ~"#version 150\n\
            in vec3 Color;\n\
            in vec2 Texcoord;\n\
            out vec4 outColor;\n\
            uniform sampler2D tex;\n\
            void main() {\n\
                outColor = texture(tex, Texcoord) * vec4(Color, 1.0);\n\
            }";

        // Create and compile the vertex shader
        let vertexShader = glCreateShader(GL_VERTEX_SHADER);
        do as_c_str(vertexSource) |data| {
            glShaderSource(vertexShader, 1, to_unsafe_ptr(&data), null());
            glCompileShader(vertexShader);
        }
        
        // Create and compile the fragment shader
        let fragmentShader = glCreateShader(GL_FRAGMENT_SHADER);
        do as_c_str(fragmentSource) |data| {
            glShaderSource(fragmentShader, 1, to_unsafe_ptr(&data), null());
            glCompileShader(fragmentShader);
        }
        
        // Link the vertex and fragment shader into a shader program
        let shaderProgram = glCreateProgram();
        glAttachShader(shaderProgram, vertexShader);
        glAttachShader(shaderProgram, fragmentShader);
        glBindFragDataLocation(shaderProgram, 0, as_c_str("outColor", |s| s));
        glLinkProgram(shaderProgram);
        glUseProgram(shaderProgram);
        
        // Specify the layout of the vertex data
        let posAttrib = glGetAttribLocation(shaderProgram, as_c_str("position", |s| s)) as GLuint;
        glEnableVertexAttribArray(posAttrib);
        glVertexAttribPointer(posAttrib, 2, GL_FLOAT, GL_FALSE,
                              7 * size_of::<GLfloat>() as GLsizei,
                              null());
        
        let colAttrib = glGetAttribLocation(shaderProgram, as_c_str("color", |s| s)) as GLuint;
        glEnableVertexAttribArray(colAttrib);
        unsafe {
            glVertexAttribPointer(colAttrib, 3, GL_FLOAT, GL_FALSE,
                                  7 * size_of::<GLfloat>() as GLsizei,
                                  transmute(2 * size_of::<GLfloat>()));
        }
        
        let texAttrib = glGetAttribLocation(shaderProgram, as_c_str("texcoord", |s| s)) as GLuint;
        glEnableVertexAttribArray(texAttrib);
        unsafe {
            glVertexAttribPointer(texAttrib, 2, GL_FLOAT, GL_FALSE,
                                  7 * size_of::<GLfloat>() as GLsizei,
                                  transmute(5 * size_of::<GLfloat>()));
        }

        // Load texture
        let tex_loaded: bool;
        let tex: GLuint = 0;
        glGenTextures(1, to_unsafe_ptr(&tex));
        
        match load_with_depth(~"resources/sample.png", 3) {
            Some(image) => {
                unsafe {
                    glTexImage2D(
                        GL_TEXTURE_2D, 0,
                        GL_RGB as GLint,
                        image.width as GLsizei,
                        image.height as GLsizei,
                        0, GL_RGB, GL_UNSIGNED_BYTE,
                        transmute(to_ptr(image.data))
                    );
                }
                
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as GLint);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as GLint);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as GLint);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as GLint);
                
                tex_loaded = true;
            }
            
            None => {
                io::println(~"Failed to load texture.");
                tex_loaded = false;
            }
        }
        
        if tex_loaded {
            while window.get_param(glfw3::CLOSE_REQUESTED) == 0 {
                // Poll events
                glfw3::poll_events();
                
                // Clear the screen to black
                glClearColor(0.1, 0.1, 0.1, 1.0);
                glClear(GL_COLOR_BUFFER_BIT);
            
                // Draw a rectangle from the 2 triangles using 6 indices
                glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, null());
                
                // Swap buffers
                window.swap_buffers();
            }
        }
        
        glDeleteTextures(1, to_unsafe_ptr(&tex));
        
        glDeleteProgram(shaderProgram);
        glDeleteShader(fragmentShader);
        glDeleteShader(vertexShader);
        
        glDeleteBuffers(1, to_unsafe_ptr(&vbo));
        
        glDeleteVertexArrays(1, to_unsafe_ptr(&vao));
        
        glfw3::terminate();
    }
}