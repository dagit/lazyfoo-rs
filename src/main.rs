extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::rect::Point;
use sdl2::event::Event;
use sdl2::render::Renderer;
use sdl2::render::Texture;
use sdl2::render::BlendMode;
use sdl2::surface::Surface;
use sdl2::pixels::Color;

const SCREEN_WIDTH  : u32 = 640;
const SCREEN_HEIGHT : u32 = 480;

const LEVEL_WIDTH  : u32 = 1280;
const LEVEL_HEIGHT : u32 = 960;

pub const DOT_WIDTH  : u32 = 20;
pub const DOT_HEIGHT : u32 = 20;
pub const DOT_VEL    : i32 = 10;

pub const TILE_WIDTH  : u32 = 80;
pub const TILE_HEIGHT : u32 = 80;
pub const TOTAL_TILES : u32 = 192;
pub const TOTAL_TILE_SPRITES : u32 = 12;

#[derive(PartialEq,Eq,Copy,Clone,Hash)]
pub enum TileSprite {
    Red,
    Green,
    Blue,
    Center,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
}

impl TileSprite {
    pub fn from_u32(v: u32) -> Option<TileSprite>
    {
        use TileSprite::*;
        match v {
            0  => Some(Red),
            1  => Some(Green),
            2  => Some(Blue),
            3  => Some(Center),
            4  => Some(Top),
            5  => Some(TopRight),
            6  => Some(Right),
            7  => Some(BottomRight),
            8  => Some(Bottom),
            9  => Some(BottomLeft),
            10 => Some(Left),
            11 => Some(TopLeft),
            _  => None,
        }
    }

    pub fn to_u32(&self) -> u32
    {
        use TileSprite::*;
        match *self {
            Red         => 0,
            Green       => 1,
            Blue        => 2,
            Center      => 3,
            Top         => 4,
            TopRight    => 5,
            Right       => 6,
            BottomRight => 7,
            Bottom      => 8,
            BottomLeft  => 9,
            Left        => 10,
            TopLeft     => 11
        }
    }
}

pub struct AppBuilder<'a>
{
    sdl_context:   Option<sdl2::Sdl>,
    renderer:      Option<Renderer<'a>>,
    image_context: Option<sdl2::image::Sdl2ImageContext>,
    tile_texture:  Option<LTexture>,
    dot_texture:   Option<LTexture>,
    tile_clips:    Vec<Rect>,
    tile_set:      Vec<Tile>,
}

impl<'a> AppBuilder<'a> {
    pub fn build(self) -> Result<App<'a>, String>
    {
        let sdl_context   = self.sdl_context.expect("Missing sdl_context");
        let renderer      = self.renderer.expect("Missing renderer");
        let image_context = self.image_context.expect("Missing image_context");
        let tile_texture  = self.tile_texture.expect("Missing tile_texture");
        let dot_texture   = self.dot_texture.expect("Missing dot_texture");
        let tile_clips    = self.tile_clips;
        let tile_set      = self.tile_set;
        Ok(App {
            sdl_context:   sdl_context,
            renderer:      renderer,
            image_context: image_context,
            tile_texture:  tile_texture,
            dot_texture:   dot_texture,
            tile_clips:    tile_clips,
            tile_set:      tile_set,
        })
    }
}
pub struct App<'a> {
    pub sdl_context:  sdl2::Sdl,
    pub renderer:      Renderer<'a>,
    pub image_context: sdl2::image::Sdl2ImageContext,
    pub tile_texture:  LTexture,
    pub dot_texture:   LTexture,
    pub tile_clips:    Vec<Rect>,
    pub tile_set:      Vec<Tile>,
}

impl<'a> App<'a> {
    pub fn new() -> AppBuilder<'a>
    {
        AppBuilder
        {
            sdl_context:   None,
            renderer:      None,
            image_context: None,
            tile_texture:  None,
            dot_texture:   None,
            tile_clips:    vec![],
            tile_set:      vec![],
        }
    }

    pub fn get_clip(&self, sprite_type: TileSprite) -> Option<Rect>
    {
        self.tile_clips.get(sprite_type.to_u32() as usize).map(|x| x.clone())
    }
}

#[derive(Copy,Clone)]
pub struct Tile {
    bounds      : Rect,
    sprite_type : TileSprite,
}

pub fn check_collision(a: &Rect, b: &Rect) -> bool
{
    //Calculate the sides of rect A
    let left_a   = a.x;
    let right_a  = a.x + a.w;
    let top_a    = a.y;
    let bottom_a = a.y + a.h;

    //Calculate the sides of rect B
    let left_b   = b.x;
    let right_b  = b.x + b.w;
    let top_b    = b.y;
    let bottom_b = b.y + b.h;

    //If any of the sides from A are outside of B
    if bottom_a <= top_b
    {
        return false
    }

    if top_a >= bottom_b
    {
        return false
    }

    if right_a <= left_b
    {
        return false
    }

    if left_a >= right_b
    {
        return false
    }

    //If none of the sides from A are outside B
    return true
}

impl Tile {
    pub fn new(x: u32, y: u32, sprite_type: TileSprite) -> Self
    {
        Self{bounds: Rect::new(x as i32, y as i32, TILE_WIDTH, TILE_HEIGHT), sprite_type: sprite_type}
    }

    fn render<'a>(&self, app: &mut App<'a>, camera: &Rect) -> Result<(), String>
    {
        let clip = app.get_clip( self.sprite_type );
        if check_collision(camera, &self.bounds) {
            app.tile_texture.render( &mut app.renderer,
                                     self.bounds.x - camera.x,
                                     self.bounds.y - camera.y,
                                     clip, 0.0f64, None)?
        }
        Ok(())
    }

    pub fn get_sprite_type(&self) -> TileSprite
    {
        self.sprite_type
    }

    pub fn get_bounds(&self) -> Rect
    {
        self.bounds
    }
}

pub struct Dot {
    bounds: Rect,
    vel_x:  i32,
    vel_y:  i32,
}

impl Dot {
    pub fn new() -> Self
    {
        Dot{ bounds: Rect::new(0, 0, DOT_WIDTH, DOT_HEIGHT)
           , vel_x:  0
           , vel_y:  0
        }
    }

    pub fn handle_event(&mut self, e: &Event)
    {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        match *e {
            Event::KeyDown {keycode: Some(code), repeat: false, ..} => {
                match code {
                    Keycode::Up    => self.vel_y -= DOT_VEL,
                    Keycode::Down  => self.vel_y += DOT_VEL,
                    Keycode::Left  => self.vel_x -= DOT_VEL,
                    Keycode::Right => self.vel_x += DOT_VEL,
                    _              => {},
                }
            },
            Event::KeyUp   {keycode: Some(code), repeat: false, ..} => {
                match code {
                    Keycode::Up    => self.vel_y += DOT_VEL,
                    Keycode::Down  => self.vel_y -= DOT_VEL,
                    Keycode::Left  => self.vel_x += DOT_VEL,
                    Keycode::Right => self.vel_x -= DOT_VEL,
                    _              => {},
                }
            },
            _ => {},
        }
    }

    pub fn move_dot(&mut self, tiles: &[Tile])
    {
        self.bounds.x += self.vel_x;

        if (self.bounds.x < 0) || (self.bounds.x as u32 + DOT_WIDTH > LEVEL_WIDTH ) ||
            touches_wall( &self.bounds, tiles )
        {
            self.bounds.x -= self.vel_x;
        }

        self.bounds.y += self.vel_y;

        if (self.bounds.y < 0) || (self.bounds.y as u32 + DOT_HEIGHT > LEVEL_HEIGHT ) ||
            touches_wall( &self.bounds, tiles )
        {
            self.bounds.y -= self.vel_y;
        }

    }

    pub fn set_camera(&self, camera: &mut Rect)
    {
        let dwidth  = DOT_WIDTH     as i32;
        let dheight = DOT_HEIGHT    as i32;
        let swidth  = SCREEN_WIDTH  as i32;
        let sheight = SCREEN_HEIGHT as i32;
        camera.x = (self.bounds.x + dwidth / 2) - swidth / 2;
        camera.y = (self.bounds.y + dheight / 2) - sheight / 2;

        if camera.x < 0
        {
            camera.x = 0;
        }
        if camera.y < 0
        {
            camera.y = 0;
        }
        if camera.x > LEVEL_WIDTH as i32 - camera.w
        {
            camera.x = LEVEL_WIDTH as i32 - camera.w;
        }
        if camera.y > LEVEL_HEIGHT as i32 - camera.h
        {
            camera.y = LEVEL_HEIGHT as i32 - camera.h;
        }
    }

    pub fn render<'a>(&self, app: &mut App<'a>, camera: &Rect) -> Result<(), String>
    {
        app.dot_texture.render( &mut app.renderer,
                                self.bounds.x - camera.x,
                                self.bounds.y - camera.y,
                                None, 0.0f64, None)
    }
}

//Texture wrapper class
pub struct LTexture
{
    //The actual hardware texture
    texture: Texture,

    //Image dimensions
    width:  u32,
    height: u32,
}

pub struct LTextureBuilder
{
    texture: Option<Texture>,
    width:   Option<u32>,
    height:  Option<u32>,
}

impl LTextureBuilder {
    pub fn new() -> Self
    {
        LTextureBuilder{texture: None, width: None, height: None}
    }

    //Loads image at specified path
    pub fn load_from_file(app: &mut AppBuilder, path: &str) -> Result<LTexture, String>
    {
        let mut tb = LTextureBuilder::new();
        let mut loaded_surface : Surface = sdl2::image::LoadSurface::from_file(path)?;
        tb.width   = Some(loaded_surface.width());
        tb.height  = Some(loaded_surface.height());
        loaded_surface.set_color_key( true, Color::RGB(0, 0xff, 0xff))?;
        let texture = app.renderer.as_ref()
            .map(|r|
                 r.create_texture_from_surface( loaded_surface )
                 .map_err(|e| format!("Unable to create texture from {} SDL Error: {}",
                                      path, e)));
        tb.texture = Some(texture.expect("Missing Renderer")
                          .expect("Failed to create texture"));
        tb.build()
    }

    pub fn build(self) -> Result<LTexture, String>
    {
        let w = self.width.expect("Missing width");
        let h = self.height.expect("Missing height");
        let t = self.texture.expect("Missing texture");
        Ok(LTexture{ texture: t, width: w, height: h })
    }
}

impl LTexture {

    //Set color modulation
    pub fn set_color(&mut self, red: u8, green: u8, blue: u8)
    {
        self.texture.set_color_mod(red, green, blue )
    }

    //Set blending
    pub fn set_blend_mode(&mut self, blending: BlendMode )
    {
        self.texture.set_blend_mode(blending)
    }

    //Set alpha modulation
    pub fn set_alpha(&mut self, alpha: u8 )
    {
        self.texture.set_alpha_mod( alpha )
    }

    //Renders texture at given point
    pub fn render<'a>(&mut self, renderer: &mut Renderer,
                      x: i32, y: i32, clip: Option<Rect>,
                      angle: f64, center: Option<Point>) -> Result<(), String>
    {
        let mut render_quad: Rect = Rect::new(x, y, self.width as u32, self.height as u32);
        clip.map(|c|{
            render_quad.w = c.w;
            render_quad.h = c.h;
        });
        renderer.copy_ex(&self.texture,
                         clip, Some(render_quad), angle, center, false, false)
    }

    //Gets image dimensions
    pub fn get_width(&self) -> u32
    {
        self.width
    }
    pub fn get_height(&self) -> u32
    {
        self.height
    }
}

fn main() {
    println!("Hello, world!");
    let mut app_builder = App::new();
    init(&mut app_builder).expect("Failed to initialize");
    load_media(&mut app_builder).expect("Failed to load media");
    let mut app = app_builder.build().expect("Failed to build app");
    let mut camera = Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut event_pump  = app.sdl_context.event_pump().unwrap();
    let mut dot = Dot::new();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{ .. } => break 'main,
                _                 => {},
            }
            dot.handle_event(&event);
        }
        dot.move_dot( &app.tile_set );
        dot.set_camera( &mut camera );

        app.renderer.set_draw_color(Color::RGBA(0xff, 0xff, 0xff, 0xff));
        app.renderer.clear();
        for i in 0..TOTAL_TILES as usize {
            let t = app.tile_set[i];
            t.render(&mut app, &camera).expect("Render failed");
        }
        dot.render(&mut app, &camera).expect("Render failed");
        app.renderer.present();
    }
}


fn init(app: &mut AppBuilder) -> Result<(), String>
{

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    // Commented this out because it doesn't appear to be fully implemented
    // in the rust bindings.
    //use sdl2::hint::set;
    //set("SDL_HINT_RENDER_SCALE_QUALITY", "1");
    let display = video_subsystem.window("Lazy Foo Lesson 39",
                                         SCREEN_WIDTH,
                                         SCREEN_HEIGHT)
        .build()
        .map_err(|err| err.to_string())?;

    let mut renderer = sdl2::render::RendererBuilder::new(display)
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|err| err.to_string())?;
    renderer.set_draw_color( Color::RGBA(0xff, 0xff, 0xff, 0xff) );

    app.renderer = Some(renderer);
    app.image_context = Some(sdl2::image::init(sdl2::image::INIT_PNG)?);
    app.sdl_context = Some(sdl_context);

    Ok(())
}

fn load_media(app: &mut AppBuilder) -> Result<(), String>
{
    app.dot_texture  = Some(LTextureBuilder::load_from_file(app, "39_tiling/dot.bmp")?);
    app.tile_texture = Some(LTextureBuilder::load_from_file(app, "39_tiling/tiles.png")?);
    set_tiles( app )
}

fn set_tiles(app: &mut AppBuilder) -> Result<(), String>
{
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::open("39_tiling/lazy.map")
        .map_err(|_| "Unable to open map file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|err| err.to_string())?;
    let tile_numbers = contents.split_whitespace()
        .map(|tile_number|
             {
                 let trimmed = tile_number.trim();
                 let tn = match trimmed.parse::<u32>() {
                     Ok(i)  => i,
                     Err(e) => {
                         println!("error parsing number");
                         return Err(e.to_string())
                     },
                 };
                 Ok(tn)
             })
        .collect::< Result< Vec<u32> , String> >()?;
    let mut x = 0;
    let mut y = 0;
    for tn in &tile_numbers {
        let tile_type = TileSprite::from_u32(*tn);
        match tile_type {
            None     => return Err(format!("Error loading map: Invalid tile type at {}!", *tn)),
            Some(tt) => {
                app.tile_set.push( Tile::new( x, y, tt ) );
                x += TILE_WIDTH;
                //If we've gone too far
                if x >= LEVEL_WIDTH
                {
                    //Move back
                    x = 0;
                    //Move to the next row
                    y += TILE_HEIGHT;
                }
            }
        }
    }
    // We don't do this with a loop, because the natural enumeration
    // order of the tile set doesn't match the order they need to be
    // here. If you get things wrong then things look really odd
    // because tiles are in the wrong spots in the map.

    // Red
    app.tile_clips.push( Rect::new(0,   0,   TILE_WIDTH, TILE_HEIGHT ) );
    // Green
    app.tile_clips.push( Rect::new(0,   80,  TILE_WIDTH, TILE_HEIGHT ) );
    // Blue
    app.tile_clips.push( Rect::new(0,   160, TILE_WIDTH, TILE_HEIGHT ) );
    // Center
    app.tile_clips.push( Rect::new(160, 80,  TILE_WIDTH, TILE_HEIGHT ) );
    // Top
    app.tile_clips.push( Rect::new(160, 0,   TILE_WIDTH, TILE_HEIGHT ) );
    // Top Right
    app.tile_clips.push( Rect::new(240, 0,   TILE_WIDTH, TILE_HEIGHT ) );
    // Right
    app.tile_clips.push( Rect::new(240, 80,  TILE_WIDTH, TILE_HEIGHT ) );
    // Bottom Right
    app.tile_clips.push( Rect::new(240, 160, TILE_WIDTH, TILE_HEIGHT ) );
    // Bottom
    app.tile_clips.push( Rect::new(160, 160, TILE_WIDTH, TILE_HEIGHT ) );
    // Bottom Left
    app.tile_clips.push( Rect::new(80,  160, TILE_WIDTH, TILE_HEIGHT ) );
    // Left
    app.tile_clips.push( Rect::new(80,  80,  TILE_WIDTH, TILE_HEIGHT ) );
    // Top Left
    app.tile_clips.push( Rect::new(80,  0,   TILE_WIDTH, TILE_HEIGHT ) );
    Ok(())
}

fn touches_wall(bounds: &Rect, tiles: &[Tile]) -> bool
{
    for i in 0..TOTAL_TILES as usize {
        use TileSprite::*;
        if (tiles[i].sprite_type.to_u32() >= Center.to_u32()) &&
            (tiles[i].sprite_type.to_u32() <= TopLeft.to_u32())
        {
            if check_collision(bounds, &tiles[i].bounds )
            {
                return true;
            }
        }
    }
    return false;
}
