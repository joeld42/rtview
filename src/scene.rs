use vec3::Vec3;

use super::ray::Ray;

pub struct Scene {

    // camera settings
    camera_center : Vec3,
    viewport_u : Vec3,
    viewport_v : Vec3,
    pixel_delta_u : Vec3,
    pixel_delta_v : Vec3,
    pixel00_loc : Vec3,
}

impl Scene {

    pub fn new( width: u32, height: u32 ) -> Scene
    {
        let fwidth = width as f32;
        let fheight = height as f32;
        let aspect = fheight / fwidth;
    
        // camera
        let focal_length  :f32   = 1.0;
        let viewport_height : f32 = 2.0;
        let viewport_width = viewport_height / aspect;
        let camera_center = Vec3::ZERO;
        println!("setup_camera: Aspect {} w {} h {}", aspect, viewport_width, viewport_height );
    
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

        // Create the scene object
        Scene {
            camera_center : camera_center,
            viewport_u : viewport_u,
            viewport_v : viewport_v,
            pixel_delta_u : pixel_delta_u,
            pixel_delta_v : pixel_delta_v,
            pixel00_loc : pixel00_loc,
        }    
    }

    pub fn ray_at_pixel_loc( &self, i : i32, j : i32 ) -> Ray 
    {
        let ii = i as f32;
        let jj = j as f32;

        let pixel_center = 
            self.pixel00_loc + (ii * self.pixel_delta_u) + (jj * self.pixel_delta_v);
        let ray_direction = pixel_center - self.camera_center;

        Ray { 
            origin : self.camera_center, 
            dir : ray_direction,
         }
    }



}
