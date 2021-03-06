extern mod glfw3;
extern mod glcore;
extern mod lmath;
extern mod numeric;
extern mod stb_image;

use cast::transmute;
use ptr::{is_null, null, to_unsafe_ptr};
use str::as_c_str;
use sys::size_of;
use vec::raw::to_ptr;

use glcore::*;
use lmath::vec::vec3::Vec3;
use lmath::mat::mat3::Mat3;
use lmath::mat::mat4::Mat4;
use numeric::types::angle::*;
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
            uniform mat4 trans;\n\
            void main() {\n\
                Color = color;\n\
                Texcoord = texcoord;\n\
                gl_Position = trans * vec4(position, 0.0, 1.0);\n\
            }";

        let fragmentSource =
          ~"#version 150\n\
            in vec3 Color;\n\
            in vec2 Texcoord;\n\
            out vec4 outColor;\n\
            uniform sampler2D texKitten;\n\
            uniform sampler2D texPuppy;\n\
            void main() {\n\
                outColor = mix(texture(texKitten, Texcoord), texture(texPuppy, Texcoord), 0.5);\n\
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
        
        // Load textures
        let textures: ~[GLuint] = ~[0, 0];
        unsafe { glGenTextures(2, to_ptr(textures)); }
        
        let kitten_loaded: bool;
        match load_with_depth(~"resources/sample.png", 3) {
            Some(image) => {
                glActiveTexture(GL_TEXTURE0);
                glBindTexture(GL_TEXTURE_2D, textures[0]);
                
                glUniform1i(glGetUniformLocation(shaderProgram, as_c_str("texKitten", |s| s)), 0);
                
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
                
                kitten_loaded = true;
            }
            None => {
                io::println(~"Failed to load kitten.");
                kitten_loaded = false;
            }
        }
        
        let puppy_loaded: bool;
        match load_with_depth(~"resources/sample2.png", 3) {
            Some(image) => {
                glActiveTexture(GL_TEXTURE1);
                glBindTexture(GL_TEXTURE_2D, textures[1]);
                
                glUniform1i(glGetUniformLocation(shaderProgram, as_c_str("texPuppy", |s| s)), 1);
                
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
                
                puppy_loaded = true;
            }
            None => {
                io::println(~"Failed to load puppy.");
                puppy_loaded = false;
            }
        }
        
        let uniTrans = glGetUniformLocation(shaderProgram, as_c_str("trans", |s| s));
        
        if kitten_loaded && puppy_loaded {
            while window.get_param(glfw3::CLOSE_REQUESTED) == 0 {
                // Poll events
                glfw3::poll_events();
                
                // Clear the screen to black
                glClearColor(0.1, 0.1, 0.1, 1.0);
                glClear(GL_COLOR_BUFFER_BIT);
                
                // Calculate transformation
                let trans: Mat4<GLfloat> = Mat3::from_axis_angle(
                    &Vec3::new(
                        0.0 as GLfloat,
                        0.0 as GLfloat,
                        1.0 as GLfloat),
                    Degrees(glfw3::get_time() as GLfloat * 180.0)
                ).to_mat4();
                
                // Set uniform to transform
                glUniformMatrix4fv(uniTrans, 1, GL_FALSE, trans.to_ptr());
            
                // Draw a rectangle from the 2 triangles using 6 indices
                glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, null());
                
                // Swap buffers
                window.swap_buffers();
            }
        }
        
        unsafe { glDeleteTextures(2, to_ptr(textures)); }
        
        glDeleteProgram(shaderProgram);
        glDeleteShader(fragmentShader);
        glDeleteShader(vertexShader);
        
        glDeleteBuffers(1, to_unsafe_ptr(&vbo));
        
        glDeleteVertexArrays(1, to_unsafe_ptr(&vao));
        
        glfw3::terminate();
    }
}