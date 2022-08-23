use notan::prelude::*;
use notan::math::{ Mat4, Quat, Vec3 };

const VERT : ShaderSource< '_ > = notan::vertex_shader!
{
  r#"
  #version 450
  
  layout( location = 0 ) in vec3 a_pos;
  layout( location = 1 ) in vec2 a_uv;
  layout( location = 0 ) out vec2 v_uv;
  
  layout( set = 1, binding = 0 ) uniform MeshTransformations
  {
    mat4 model;
  };
  
  void main()
  {
    v_uv = a_uv;
    gl_Position = model * vec4( a_pos.x, a_pos.y * -1.0, a_pos.z, 1.0 );
  }
  "#
};

const FRAG : ShaderSource< '_ > = notan::fragment_shader!
{
  r#"
  #version 450
  precision mediump float;
  
  layout( location = 0 ) in vec2 v_uv;
  layout( location = 0 ) out vec4 color;
  layout( location = 0 ) uniform sampler2D u_texture;
  
  void main()
  {
    color = texture( u_texture, v_uv );
  }
  "#
};

#[ derive( Debug ) ]
pub struct Mesh
{
  pub texture : Asset< Texture >,
  pub vertext_buffer : Buffer,
  pub index_buffer : Buffer,
  pub transformations_buffer : Buffer,
  pub transformations : Mat4,
}

impl Mesh
{
  pub fn new( gfx: &mut Graphics, assets : &mut Assets, path : &'static str, scale : impl Into< Vec3 >, translation : impl Into< Vec3 > ) -> Self
  {
    let texture = assets.load_asset( path ).unwrap();
    
    let vertex_info = VertexInfo::new()
    .attr( 0, VertexFormat::Float32x3 ) // positions
    .attr( 1, VertexFormat::Float32x2 ); // uvs

    let vertices =
    [
    -1.0,  -1.0,  0.0,    0.0, 0.0,
    -1.0,   1.0,  0.0,    0.0, 1.0,
    1.0,   1.0,  0.0,    1.0, 1.0,
    1.0,  -1.0,  0.0,    1.0, 0.0,
    ];
    let vertext_buffer = gfx
    .create_vertex_buffer()
    .with_info( &vertex_info )
    .with_data( &vertices )
    .build().unwrap();
    
    let indices = [0, 1, 2, 2, 3, 0];
    let index_buffer = gfx
    .create_index_buffer()
    .with_data( &indices )
    .build().unwrap();
    
    let transformations = Mat4::from_scale_rotation_translation( scale.into(), Quat::IDENTITY, translation.into() );
    let transformations_buffer = gfx.create_uniform_buffer( 1, "MeshTransformations" )
    .build().unwrap();

    Self 
    {
      texture,
      vertext_buffer,
      index_buffer,
      transformations_buffer,
      transformations,
    }
  }

  fn draw_texture( &self, pipeline: &Pipeline, gfx : &mut Graphics) 
  {
    let mut renderer = gfx.create_renderer();

    renderer.begin( None );
    renderer.set_pipeline( pipeline );
    
    if let Some( texture ) = &self.texture.lock()
    {
      renderer.bind_texture( 0, texture );
    }
    renderer.bind_buffers( &[ &self.vertext_buffer, &self.index_buffer, &self.transformations_buffer ] );
    renderer.draw( 0, 6 );
    renderer.end();
    gfx.render( &renderer );
  }
}

#[ derive( AppState, Debug ) ]
pub struct Scene
{
  pipeline : Pipeline,
  meshes : Vec< Mesh >,
}

impl Scene
{
  fn new( assets : &mut Assets, gfx : &mut Graphics ) -> Scene
  {
    let depth_test = DepthStencil
    {
      write : true,
      compare : CompareMode::Less,
    };
    
    let vertex_info = VertexInfo::new()
    .attr( 0, VertexFormat::Float32x3 ) // positions
    .attr( 1, VertexFormat::Float32x2 ); // uvs
    
    let pipeline = gfx
    .create_pipeline()
    .from( &VERT, &FRAG )
    .with_color_blend( BlendMode::NORMAL )
    .with_vertex_info( &vertex_info )
    .with_texture_location( 0, "u_texture" )
    .with_depth_stencil( depth_test )
    .build()
    .unwrap();

    let meshes = vec![
      Mesh::new(gfx, assets, "./assets/icon_ethenium.png", ( 0.1, 0.11, 0.1 ), ( -0.11, -0.01, -0.04 )),
      Mesh::new(gfx, assets, "./assets/icon_voice.png", ( 0.09, 0.09, 0.1 ), ( -0.026, -0.0025, 0.0012 ))
    ];
    
    Scene
    {
      pipeline,
      meshes,
    }
  }
  
  fn render( gfx : &mut Graphics, scene : &mut Self )
  {
    for mesh in &mut scene.meshes
    {
      gfx.set_buffer_data( &mesh.transformations_buffer, &mesh.transformations.to_cols_array() );
      mesh.draw_texture( &scene.pipeline, gfx );
    }
  }
}

#[ wasm_bindgen::prelude::wasm_bindgen ]
pub fn main() -> Result< (), String >
{
  let window_config = WindowConfig::default().transparent();
  notan::init_with( Scene::new )
  .add_config( window_config )
  .draw( Scene::render )
  .build()
}
