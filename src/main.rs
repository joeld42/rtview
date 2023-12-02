use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use slint::SharedPixelBuffer;
use slint::Rgb8Pixel;
use slint::Image;
use slint::Timer;
use slint::TimerMode;

use vec3::Vec3;

struct Ray {
    origin : Vec3,
    dir : Vec3,
}

impl Ray {

    fn at( &self, t : f32 ) -> Vec3 {
        self.origin + (self.dir * t)
    }
}

enum TileStatus {
    Clear, // Reset or clear the tile
    Rendering,
    Denoising,
    Finished
} 

struct Tile {
    x : u32,
    y : u32,
    w : u32,
    h : u32,
    status : TileStatus,
    pixels : Option<SharedPixelBuffer<Rgb8Pixel>>,
}



fn mk_col32( r : f32, g : f32, b : f32 ) -> u32 {
    let r = ((r * 255.0) as u32) & 0xff;
    let g = ((g * 255.0) as u32) & 0xff;
    let b = ((b * 255.0) as u32) & 0xff;
    
    // pack the colors into a u32
    (r << 16) | (g << 8) | b
}

fn ray_color( ray: &Ray ) -> Vec3 {

    let t = hit_sphere( &Vec3::new(0.0, 0.0, -1.0), 0.5, ray);
    if t > 0.0 { 

        let N = Vec3::normalize( ray.at(t) - Vec3::new( 0.0, 0.0, -1.0 ) );
        return 0.5 * Vec3::new( N.x+1.0, N.y+1.0, N.z+1.0 );   
    }

    let unit_dir = ray.dir.normalize();    
    let a = 0.5 * unit_dir.y + 1.0;
    
    (1.0-a)*Vec3::ONE + a*Vec3::new( 0.5, 0.7, 1.0)
}

fn hit_sphere( center : &Vec3, radius : f32, ray : &Ray ) -> f32 {
    let oc = ray.origin - center;
    let a = Vec3::dot( &ray.dir, &ray.dir );
    let b = 2.0 * Vec3::dot( &oc, &ray.dir );
    let c = Vec3::dot( &oc, &oc ) - radius*radius;
    let discriminant = b*b - 4.0*a*c;

    
    if discriminant < 0.0 {
        return -1.0;
    } else {
        return (-b - discriminant.sqrt()) / (2.0*a);
    }
}

fn do_render(width: u32, height: u32, buffer: &mut [u8]) {
    

    let fwidth = width as f32;
    let fheight = height as f32;
    let aspect = fheight / fwidth;

    // camera
    let focal_length  :f32   = 1.0;
    let viewport_height : f32 = 2.0;
    let viewport_width = viewport_height / aspect;
    let camera_center = Vec3::ZERO;
    println!("Aspect {} w {} h {}", aspect, viewport_width, viewport_height );

    let viewport_u = Vec3::new( viewport_width, 0.0, 0.0 );
    let viewport_v = Vec3::new( 0.0, -viewport_height, 0.0);

    // upper left
    let viewport_upper_left = camera_center
                        - Vec3::new( 0.0,0.0, focal_length )
                        - viewport_u/2.0
                        - viewport_v/2.0;
    let pixel_delta_u = viewport_u / fwidth;
    let pixel_delta_v = viewport_v / fheight;

    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);


    for j in 0..height {
        let jj = j as f32;
        //println!("row {j}");        
        for i in 0..width {
            let ii = i as f32;
            let ndx : usize = usize::try_from((j*width+i) * 3).unwrap();

            let pixel_center = 
                        pixel00_loc + (ii * pixel_delta_u) + (jj * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;

            let ray = Ray { 
                            origin : camera_center, 
                            dir : ray_direction,
                         };

            let col = ray_color( &ray );


            buffer[ndx+0] = (col.x * 255.0) as u8;
            buffer[ndx+1] = (col.y * 255.0) as u8;
            buffer[ndx+2] = (col.z * 255.0) as u8;
        }
    }
    
}

// todo: replace with better
fn rand_hash( x : f32, y : f32, z : f32 ) -> f32{
    (Vec3::dot( 
        &Vec3::new( x,y,z ),
        &Vec3::new( 1.0, 113.0, 21.5) ).sin() * 43758.5453123 ).fract()
}

fn render_tile( tile : &mut Tile ) {
    let mut tile_px = SharedPixelBuffer::<Rgb8Pixel>::new( tile.w, tile.h);
    
    let buffer = tile_px.make_mut_bytes();

    // "random" color per tile
    // return fract(sin(dot(p, vec2(1.0,113.0)))*43758.5453123);

    let col = Vec3::new( 
        rand_hash(tile.x as f32, tile.y as f32, 0.0),
        rand_hash(tile.x as f32, tile.y as f32, 1.0),
        rand_hash(tile.x as f32, tile.y as f32, 2.0) );

    for j in 0..tile.h {                
        for i in 0..tile.w {

            let ndx : usize = usize::try_from((j*tile.w+i) * 3).unwrap();
            

            buffer[ndx+0] = (col.x * 255.0) as u8;
            buffer[ndx+1] = (col.y * 255.0) as u8;
            buffer[ndx+2] = (col.z * 255.0) as u8;
        }
    }

    tile.pixels = Some( tile_px );

    // todo return Result
}

fn ceil_div( a : u32, b : u32 ) -> u32 {
    (a + b - 1 ) / b
}

fn main() {

    //MainWindow::new().unwrap().run().unwrap();
    let main_window = MainWindow::new().unwrap();

    let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(320, 200);

    let tile_sz = 32;
    let W = pixel_buffer.width();
    let H = pixel_buffer.height();
    let num_tiles_x = ceil_div( W, tile_sz);
    let num_tiles_y = ceil_div(H, tile_sz);
    println!("Num tiles {} x {} img {} x {}", num_tiles_x, num_tiles_y, 
                num_tiles_x * tile_sz, num_tiles_y * tile_sz );

    // do_render(pixel_buffer.width(), pixel_buffer.height(),
    //                 pixel_buffer.make_mut_bytes());


    let image = Image::from_rgb8(pixel_buffer.clone() );
    main_window.set_render_img( image );

    // queue for tiles to render
    let (tx_todo_tiles, rx_todo_tiles ) = mpsc::channel();

    for tj in 0..num_tiles_y {
        for ti in 0..num_tiles_x {
            let tile = Tile {
                    x : ti * tile_sz,
                    y : tj * tile_sz,
                    w : std::cmp::min( tile_sz, W.checked_sub( (ti+0)*tile_sz).unwrap_or(0) ),
                    h : std::cmp::min( tile_sz, H.checked_sub( (tj+0)*tile_sz).unwrap_or(0) ),
                    status: TileStatus::Clear,
                    pixels : None
            };

            tx_todo_tiles.send( tile );
        }
    }

    // queue for finished tiles
    let (tx_done_tiles, rx_done_tiles) = mpsc::channel();

    // Wrap the todo channel in an Arc and a Mutex
    let rx_todo_tiles = Arc::new(Mutex::new(rx_todo_tiles));

    // spawn a thread to render the tiles
    let num_threads = 8;
    for i in 0..num_threads {
        let tx_done_tiles2 = tx_done_tiles.clone();

        let rx_todo_clone = Arc::clone(&rx_todo_tiles);
        
        thread::spawn( move || {

            let mut rx_todo = rx_todo_clone.lock().unwrap();

            //for tile in rx_todo_tiles2 {
            while let Ok(tile) = rx_todo.recv() {

                drop(rx_todo); // release the mutex

                println!("Got tile {} {} in render thread", tile.x, tile.y );

                // render tile
                // TODO: reuse tile?
                let mut rndrTile = Tile { 
                    status : TileStatus::Finished,
                    x : tile.x, 
                    y : tile.y,
                    w : tile.w,
                    h : tile.h,
                    pixels: None
                };

                render_tile( &mut rndrTile );

                thread::sleep(std::time::Duration::from_millis(250));      
                //println!( $"Rendered tile, pixels is {")
                
                tx_done_tiles2.send( rndrTile );

                // reaquire the lock on the todo list
                rx_todo = rx_todo_clone.lock().unwrap();
            }        
            print!("Thread {i} finished...");
        });
    }


    

    // Set up a timer to update the tiles
    let ui_handle = main_window.as_weak();
    let timer = Timer::default();
    timer.start( TimerMode::Repeated, 
        std::time::Duration::from_millis(200), move || {
            let ui = ui_handle.unwrap();
            println!("Ping from timer...");

            //for tile in rx_done_tiles {
            //    println!("got tile {}, {}", tile.x, tile.y );
            //}    

            
                
            //let tile = rx_done_tiles.try_recv();            
            //println!("Got tile {}, {} in update", tile.x, tile.y );
            
            while let Ok(tile) = rx_done_tiles.try_recv() {

                if (tile.pixels.is_some() )
                {
                    let upd_buffer = pixel_buffer.clone();                
                    let stride = upd_buffer.width();
                    let buff = pixel_buffer.make_mut_bytes();

                    let mut tileImg = tile.pixels.unwrap();
                    let tw = tileImg.width();
                    let th = tileImg.height();
                    let tilebuf = tileImg.make_mut_bytes();

                    // for ij copy tilebug into buff
                    println!("draw tile at {}, {} sz {} {}", tile.x, tile.y, tile.w, tile.h );
                    for j in 0..th {
                        for i in 0..tw {
                            let ndx: usize = ((((stride * (j+tile.y)) + (i+tile.x))) * 3) as usize;
                            let tile_ndx: usize = ((j * tw + i) * 3 + 0) as usize;

                            buff[ ndx + 0 ] = tilebuf[  tile_ndx + 0 ];
                            buff[ ndx + 1 ] = tilebuf[  tile_ndx + 1 ];
                            buff[ ndx + 2 ] = tilebuf[  tile_ndx + 2 ];

                        }
                    }

                    let image = Image::from_rgb8( upd_buffer );
                    // let image = Image::from_rgb8( tile.pixels.unwrap() );
                    ui.set_render_img( image );      
                }
            }                  
        });



    main_window.run().unwrap();
}

slint::slint! {
    export component MainWindow inherits Window {
        in property render-img <=> render.source;

        render := Image {
            // width: 320px;
            // height: 200px;
        }
    }
}
