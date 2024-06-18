pub const MESH_SHADER_VS: &str = r#"
    #version 330
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec3 aColor;

    out vec3 outColor;
    // layout (location = 3) in mat4 instanceMatrix;

    uniform mat4 proj; 
    uniform mat4 view;
    uniform mat4 model;

    uniform vec3 chunkPos;

    void main() {
        gl_Position = proj * view * vec4(aPos + chunkPos, 1.0);
        outColor = aColor;
    }
"#;

pub const MESH_SHADER_FS: &str = r#"
    #version 330

    out vec4 frag_color;

    in vec3 outColor;

    void main() {
        frag_color = vec4(outColor, 1.0);
    }
"#;

pub const LINGERING_SHADER_VS: &str = r#"
    #version 330 core
    layout (location = 0) in vec2 aPos;
    layout (location = 1) in vec2 aTexCoords;

    out vec2 TexCoords;

    void main()
    {
        TexCoords = aTexCoords;
        gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0); 
    } 
"#;

pub const LINGERING_SHADER_FS: &str = r#"
    #version 330 core
    out vec4 FragColor;

    in vec2 TexCoords;

    uniform sampler2D screenTexture;

    void main()
    {
        vec3 col = texture(screenTexture, TexCoords * 1.0).rgb;
        FragColor = vec4(col, 0.1);
    } 
"#;

pub const GRAPH_SHADER_VS: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;

    uniform float ofsx;
    uniform float ofsy;

    out float ypos;

    void main()
    {
        ypos = ofsy;
        gl_Position = vec4(aPos + vec3(ofsx, ofsy, 0), 1.0); 
    }
"#;

pub const GRAPH_SHADER_FS: &str = r#"
    #version 330 core
    out vec4 FragColor;

    in float ypos;
    uniform float ofsx;

    void main()
    {
        FragColor = vec4(ypos*ypos, ypos+.1, ofsx, 1);
    } 
"#;

pub const CLR_GRAPH_SHADER_VS: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;

    void main()
    {
        gl_Position = vec4(aPos, 1.0); 
    }
"#;

pub const CLR_GRAPH_SHADER_FS: &str = r#"
    #version 330 core
    out vec4 FragColor;

    void main()
    {
        FragColor = vec4(0.1);
    } 
"#;
