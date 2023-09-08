use core::slice;
use std::collections::HashMap;
use std::process::exit;
use std::time::Instant;
use mimalloc::MiMalloc;

#[cfg(not(feature = "dhat-heap"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;



// extern crate fxhash;
use rustc_hash::FxHashMap;
use micromath::F32;
use micromath::vector::{F32x2, I8x2, I32x2};
use speedy2d::color::Color;



use speedy2d::shape::Rectangle;
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::{Graphics2D, Window};
use turborand::prelude::*;

use rayon::prelude::*;

#[cfg(debug_assertions)]
const BOID_COUNT: usize = 500;
// if in release mode
#[cfg(not(debug_assertions))]
const BOID_COUNT: usize = 10000;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;


const ARENA_SIZE: f32 = 120.0;
use std::env;

// #[cfg(not(debug_assertions))]
// use quit;

const VEC_ARENA_SIZE: F32x2 = F32x2{
    x: ARENA_SIZE,
    y: ARENA_SIZE
};

// higher numbers = strengenthes effect of group spaceing
const BUNCH_UP: f32 = 3.1;

const SPEED:f32= 0.6;

const BOID_SIZE : f32 = 1.0;

const INERTIA: f32 = 25.1;

const POINT_COUNT: usize = 500;

const RANDOMNESS: f32 = 8.1;

const GROUP_SIZE: i32 = 8;

const GROUP_SPACING: f32 = 8.1;

const GROUP_MOVEMENT: f32 = 3.4;


const CENTER_ATTRACTION: f32 = 4.1;

const VEC_GROUP_SPACING: F32x2 = F32x2{
    x: GROUP_SPACING,
    y: GROUP_SPACING
};



#[derive(Clone, Copy)]
struct Boid {
    position: F32x2,
    velocity: I8x2,
    current_point: I32x2
}

impl Boid {
    fn new() -> Boid {
        Boid {
            position: F32x2{x:ARENA_SIZE/2.0, y:ARENA_SIZE/2.0},
            velocity: I8x2{x:0,y:0},
            current_point: I32x2 {x: -1, y:-1},
        }
    }
}



fn main() {

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let rand = Rng::new();
    let window = Window::new_centered(format!("Boids {}", BOID_COUNT), (650, 640)).unwrap();
    let map:FxHashMap<(i32,i32), Vec<usize>> = FxHashMap::default();
    

    let mut boids = vec![Boid::new(); BOID_COUNT];
    boids.shrink_to_fit();

    window.run_loop(MyWindowHandler {
        // frame: Instant::now(),
        boids: boids,
        rand: rand,
        points: map,
    });
}

struct MyWindowHandler {
    // frame: Instant,
    boids: Vec<Boid>,
    rand: Rng,
    points: FxHashMap<(i32,i32), Vec<usize>>,
}

impl WindowHandler for MyWindowHandler {
    // #[inline(always)]
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        // helper.set_resizable(false);
        graphics.clear_screen(Color::from_rgb(0.8, 0.9, 1.0));
        // graphics.draw_line(
        //     (0.0, 0.0),
        //     (2.0 * self.frame.elapsed().as_millis() as f32, 0.0),
        //     4.0,
        //     Color::BLACK,
        // );
        

        
        graphics.draw_rectangle(
            Rectangle::from_tuples(
                (20.0, 20.0),
                (20.0 + ARENA_SIZE * 5.0, 20.0 + ARENA_SIZE * 5.0),
            ),
            Color::WHITE,
        );





        let mut new_values: HashMap<(i32,i32), Vec<usize>> = HashMap::new();


        // self.frame = Instant::now();

        let mut boid = Boid::new();
        let mut deflect;
        let mut nearby = vec![];
        let mut list = vec![];

        // create slice of self.boids

        let boids_slice:&mut [Boid] = self.boids.as_mut_slice();

        // graphics.draw_rectangle(Rectangle::from_tuples((0.0,0.0), bottom_right), color)
        
        for i in 0..BOID_COUNT {

            // #[cfg(not(debug_assertions))]{assert!(i < boids_slice.len());}
            
            boid.clone_from(&boids_slice[i]);
            // let mut boid = self.boids[i].clone();
            let mut v:F32x2= boid.velocity.into();
            v *= INERTIA;

            // vx = boid.velocity.0 as f32 * INERTIA;
            // vy = boid.velocity.1 as f32 * INERTIA;




            // get pixel color at current position from texture (an array of &[u8])

            deflect = F32x2{x:0.0,y:0.0};
            nearby.clear();
            let here = I32x2{
                    x:((boid.position.x/ARENA_SIZE)*POINT_COUNT as f32)as i32,
                    y:((boid.position.y/ARENA_SIZE)*POINT_COUNT as f32)as i32,
            };
            
            for i in [(0_i32,0_i32), (-1_i32,0_i32), (1_i32,0_i32), (0_i32,1_i32), (0_i32,-1_i32)] {
                if !self.points.contains_key(&(
                    ((boid.position.x/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.0,
                    ((boid.position.y/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.1,
                )) {
                    continue;
                }
                // assert!(self.points.contains_key(&(
                //     ((boid.position.x/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.0,
                //     ((boid.position.y/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.1,
                // )));
                nearby.extend_from_slice(
                    self.points.get(&(
                    ((boid.position.x/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.0,
                    ((boid.position.y/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.1,
                    )).unwrap().as_slice()
            );
                
                nearby.truncate(GROUP_SIZE as usize * 4);
                
            }
            let mut average = F32x2{x:0.0,y:0.0};
            // // let average = nearby.iter().map(|x|boids_slice[*x].position).sum();
            
            for (i,j) in nearby.iter().enumerate() {
                
                #[cfg(not(debug_assertions))]{assert!(*j < boids_slice.len())};
                average += boids_slice[*j].position;
                if i < GROUP_SIZE as usize {
                let other = &boids_slice[*j];
                let dx = (other.position.x - boid.position.x).abs();
                let dy = (other.position.y - boid.position.y).abs();
                if dx < GROUP_SPACING*2.0 && dy < GROUP_SPACING*2.0 {
                    deflect += other.velocity.into()
                }
                }
            }
            
            
            
            // for j in nearby.iter().take(GROUP_SIZE as usize) {
            // }
            average *= 1.0/nearby.len() as f32;
            
            v += (VEC_GROUP_SPACING-(average - boid.position)) * BUNCH_UP;
            // v.y +=  (GROUP_SPACING-(average.x - boid.position.x)) * BUNCH_UP;
            
            // vx -= (20.0-(average.0 - boid.position.x)) * FOLLOW;
            // vy -= (20.0-(average.1 - boid.position.y)) * FOLLOW;
            
            
            v += (VEC_ARENA_SIZE*0.5 - boid.position) * CENTER_ATTRACTION;
            // v.y += (ARENA_SIZE/2.0 - boid.position.y) * CENTER_ATTRACTION;


            v += deflect * GROUP_MOVEMENT;
            // v.y += deflect.y as f32 * GROUP_MOVEMENT;
            
            


            

            v.x += (self.rand.gen_i8() / 10) as f32 * RANDOMNESS;
            v.y += (self.rand.gen_i8() / 10) as f32 * RANDOMNESS;

            

            let velocity_length = (v.x.powi(2) + v.y.powi(2)).sqrt();
            let v =v* (1.0/velocity_length);
            // boid.velocity.0 = (vx * 100.0) as i8;
            // boid.velocity.1 = (vy * 100.0) as i8;

            boid.velocity = I8x2 {
                x:(v.x * 100.0) as i8,
                y:(v.y * 100.0) as i8,
            };

            // println!("{:?}", boid.velocity);


            let vel: F32x2 = boid.velocity.into();
            boid.position += vel * (1.0/100.0) * SPEED;
            boid.position.x = boid.position.x.min(ARENA_SIZE).max(0.0);
            boid.position.y = boid.position.y.min(ARENA_SIZE).max(0.0);

            if boid.position.x == ARENA_SIZE || boid.position.x == 0.0 {
                boid.velocity.x *= -1;
            }
            if boid.position.y == ARENA_SIZE || boid.position.y == 0.0 {
                boid.velocity.y *= -1;
            }

            // if boid.position.x == ARENA_SIZE || boid.position.x == 0.0 {
            //     boid.velocity.x *= -1;
            // }
            // if boid.position.y == ARENA_SIZE || boid.position.y == 0.0 {
            //     boid.velocity.y *= -1;
            // }

            if new_values.contains_key(&(here.x,here.y)) {
                new_values.get_mut(&(here.x,here.y)).unwrap().push(i);
            } else {
                new_values.insert((here.x,here.y), vec![i]);
            }

            // println!("{:?}",v);

            if boid.current_point != here {
                list.clear();
                list = self.points.get_mut(&(boid.current_point.x,boid.current_point.y)).unwrap_or(&mut list).to_vec();
                if let Some(list) = self.points.get_mut(&(boid.current_point.x,boid.current_point.y)) {
                    list.retain(|x| x!=&i);
                }
                if let Some(new) = self.points.get_mut(&(here.x,here.y)) {
                    new.push(i);
                    boid.current_point = here;
                }else {
                    self.points.insert((here.x,here.y), vec![i]);
                    boid.current_point = here;
                }
            }

            boids_slice[i] = boid;
            
            
            
            graphics.draw_rectangle(Rectangle::from_tuples(((boid.position.x)*5.0+20.0-BOID_SIZE/2.0,(boid.position.y)*5.0+20.0-BOID_SIZE/2.0), ((boid.position.x)*5.0+20.0+BOID_SIZE/2.0,(boid.position.y)*5.0+20.0+BOID_SIZE/2.0)), Color::BLUE);

        }


        

        
        // self.points.clear();

        // mem::swap(&mut self.points, &mut new_values);
            

    
        helper.request_redraw();
    }

    #[cfg(feature = "dhat-heap")]
    fn on_keyboard_char(
        &mut self,
        helper: &mut WindowHelper<()>,
        unicode_codepoint: char
    ) {
        // kill program and drop all values
        
        if unicode_codepoint == 'q' {

            helper.terminate_loop();
            // helper.
        }
    }

    
}
