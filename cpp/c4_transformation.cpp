// Link statically with GLEW
#define GLEW_STATIC

// Headers
#include <GL/glew.h>
#include <SFML/Window.hpp>
#include <iostream>
#include <ctime>
#include <SOIL.h>
#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/type_ptr.hpp>

// Shader sources
const char* vertexSource =
    "#version 150\n"
    "in vec2 position;"
    "in vec3 color;"
    "in vec2 texcoord;"
    "out vec3 Color;"
    "out vec2 Texcoord;"
    "uniform mat4 trans;"
    "void main() {"
    "   Color = color;"
    "   Texcoord = texcoord;"
    "   gl_Position = trans * vec4( position, 0.0, 1.0 );"
    "}";
const char* fragmentSource =
    "#version 150\n"
    "in vec3 Color;"
    "in vec2 Texcoord;"
    "out vec4 outColor;"
    "uniform sampler2D texKitten;"
    "uniform sampler2D texPuppy;"
    "void main() {"
    "   outColor = mix( texture( texKitten, Texcoord ), texture( texPuppy, Texcoord ), 0.5 );"
    "}";

int main()
{
    sf::Window window( sf::VideoMode( 800, 600, 32 ), "OpenGL", sf::Style::Titlebar | sf::Style::Close );
    
    // Initialize GLEW
    glewExperimental = GL_TRUE;
    glewInit();

    // Create Vertex Array Object
    GLuint vao;
    glGenVertexArrays( 1, &vao );
    glBindVertexArray( vao );

    // Create a Vertex Buffer Object and copy the vertex data to it
    GLuint vbo;
    glGenBuffers( 1, &vbo );

    float vertices[] = {
    //  Position   Color             Texcoords
        -0.5f,  0.5f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f, // Top-left
         0.5f,  0.5f, 0.0f, 1.0f, 0.0f, 1.0f, 0.0f, // Top-right
         0.5f, -0.5f, 0.0f, 0.0f, 1.0f, 1.0f, 1.0f, // Bottom-right
        -0.5f, -0.5f, 1.0f, 1.0f, 1.0f, 0.0f, 1.0f  // Bottom-left
    };

    glBindBuffer( GL_ARRAY_BUFFER, vbo );
    glBufferData( GL_ARRAY_BUFFER, sizeof( vertices ), vertices, GL_STATIC_DRAW );

    // Create an element array
    GLuint ebo;
    glGenBuffers( 1, &ebo );

    GLuint elements[] = {
        0, 1, 2,
        2, 3, 0
    };

    glBindBuffer( GL_ELEMENT_ARRAY_BUFFER, ebo );
    glBufferData( GL_ELEMENT_ARRAY_BUFFER, sizeof( elements ), elements, GL_STATIC_DRAW );

    // Create and compile the vertex shader
    GLuint vertexShader = glCreateShader( GL_VERTEX_SHADER );
    glShaderSource( vertexShader, 1, &vertexSource, NULL );
    glCompileShader( vertexShader );

    // Create and compile the fragment shader
    GLuint fragmentShader = glCreateShader( GL_FRAGMENT_SHADER );
    glShaderSource( fragmentShader, 1, &fragmentSource, NULL );
    glCompileShader( fragmentShader );

    // Link the vertex and fragment shader into a shader program
    GLuint shaderProgram = glCreateProgram();
    glAttachShader( shaderProgram, vertexShader );
    glAttachShader( shaderProgram, fragmentShader );
    glBindFragDataLocation( shaderProgram, 0, "outColor" );
    glLinkProgram( shaderProgram );
    glUseProgram( shaderProgram );

    // Specify the layout of the vertex data
    GLint posAttrib = glGetAttribLocation( shaderProgram, "position" );
    glEnableVertexAttribArray( posAttrib );
    glVertexAttribPointer( posAttrib, 2, GL_FLOAT, GL_FALSE, 7 * sizeof( float ), 0 );

    GLint colAttrib = glGetAttribLocation( shaderProgram, "color" );
    glEnableVertexAttribArray( colAttrib );
    glVertexAttribPointer( colAttrib, 3, GL_FLOAT, GL_FALSE, 7 * sizeof( float ), (void*)( 2 * sizeof( float ) ) );

    GLint texAttrib = glGetAttribLocation( shaderProgram, "texcoord" );
    glEnableVertexAttribArray( texAttrib );
    glVertexAttribPointer( texAttrib, 2, GL_FLOAT, GL_FALSE, 7 * sizeof( float ), (void*)( 5 * sizeof( float ) ) );

    // Load textures
    GLuint textures[2];
    glGenTextures( 2, textures );
    
    int width, height;
    unsigned char* image;

    glActiveTexture( GL_TEXTURE0 );
    glBindTexture( GL_TEXTURE_2D, textures[0] );
        image = SOIL_load_image( "sample.png", &width, &height, 0, SOIL_LOAD_RGB );
        glTexImage2D( GL_TEXTURE_2D, 0, GL_RGB, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, image );
        SOIL_free_image_data( image );
    glUniform1i( glGetUniformLocation( shaderProgram, "texKitten" ), 0 );

    glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE );
    glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE );
    glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR );
    glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR );

    glActiveTexture( GL_TEXTURE1 );
    glBindTexture( GL_TEXTURE_2D, textures[1] );
        image = SOIL_load_image( "sample2.png", &width, &height, 0, SOIL_LOAD_RGB );
        glTexImage2D( GL_TEXTURE_2D, 0, GL_RGB, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, image );
        SOIL_free_image_data( image );
    glUniform1i( glGetUniformLocation( shaderProgram, "texPuppy" ), 1 );

    glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE );
    glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE );
    glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR );
    glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR );

    GLint uniTrans = glGetUniformLocation( shaderProgram, "trans" );

    while ( window.IsOpened() )
    {
        sf::Event windowEvent;
        while ( window.GetEvent( windowEvent ) )
        {
            switch ( windowEvent.Type )
            {
            case sf::Event::Closed:
                window.Close();
                break;
            }
        }

        // Clear the screen to black
        glClearColor( 0.0f, 0.0f, 0.0f, 1.0f );
        glClear( GL_COLOR_BUFFER_BIT );

        // Calculate transformation
        glm::mat4 trans;
        trans = glm::rotate(
            trans,
            (float)clock() / (float)CLOCKS_PER_SEC * 180.0f,
            glm::vec3( 0.0f, 0.0f, 1.0f )
        );
        glUniformMatrix4fv( uniTrans, 1, GL_FALSE, glm::value_ptr( trans ) );
        
        // Draw a rectangle from the 2 triangles using 6 indices
        glDrawElements( GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0 );

        // Swap buffers
        window.Display();
    }

    glDeleteTextures( 2, textures );

    glDeleteProgram( shaderProgram );
    glDeleteShader( fragmentShader );
    glDeleteShader( vertexShader );

    glDeleteBuffers( 1, &ebo );
    glDeleteBuffers( 1, &vbo );

    glDeleteVertexArrays( 1, &vao );
}