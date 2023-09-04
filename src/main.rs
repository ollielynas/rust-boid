use core::slice;
use std::collections::HashMap;
use std::time::Instant;
use mimalloc::MiMalloc;

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
const BOID_COUNT: usize = 1000;
// if in release mode
#[cfg(not(debug_assertions))]
const BOID_COUNT: usize = 50000;



const ARENA_SIZE: f32 = 120.0;

const VEC_ARENA_SIZE: F32x2 = F32x2{
    x: ARENA_SIZE,
    y: ARENA_SIZE
};

const BUNCH_UP: f32 = 5.6;

const SPEED:f32= 0.6;

const BOID_SIZE : f32 = 1.0;

const INERTIA: f32 = 25.1;

const POINT_COUNT: usize = 100;

const RANDOMNESS: f32 = 40.1;

const GROUP_SIZE: i32 = 15;

const GROUP_SPACING: f32 = 1.1;

const GROUP_MOVEMENT: f32 = 3.8;

// const VEC_GROUP_MOVEMENT: F32x2 = F32x2{
//     x:GROUP_MOVEMENT,
//     y:GROUP_MOVEMENT
// };

const CENTER_ATTRACTION: f32 = 2.1;

const VEC_GROUP_SPACING: F32x2 = F32x2{
    x: GROUP_SPACING,
    y: GROUP_SPACING
};


// const ARENA_SIZE: f32 = 100.0;

// const BUNCH_UP: f32 = 3.0;

// const SPEED:f32= 0.3;

// const BOID_SIZE : f32 = 1.0;

// const INERTIA: f32 = 22.1;

// const POINT_COUNT: usize = 100;

// const RANDOMNESS: f32 = 20.1;

// const GROUP_SIZE: i32 = 69;

// const GROUP_SPACING: f32 = 0.01;

// const GROUP_MOVEMENT: f32 = 29.8;

// const CENTER_ATTRACTION: f32 = 17.1;



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
    let rand = Rng::new();
    let window = Window::new_centered("Title", (650, 640)).unwrap();
    let map:FxHashMap<(i32,i32), Vec<usize>> = FxHashMap::default();
    let boids = vec![Boid::new(); BOID_COUNT];
    window.run_loop(MyWindowHandler {
        frame: Instant::now(),
        boids: boids,
        rand: rand,
        points: map,
    });
}

struct MyWindowHandler {
    frame: Instant,
    boids: Vec<Boid>,
    rand: Rng,
    points: FxHashMap<(i32,i32), Vec<usize>>,
}

impl WindowHandler for MyWindowHandler {
    #[inline(always)]
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        // helper.set_resizable(false);
        graphics.set_clip(None);
        graphics.clear_screen(Color::from_rgb(0.8, 0.9, 1.0));
        graphics.draw_line((0.0, 0.0), (32.0, 0.0), 4.0, Color::GREEN);
        graphics.draw_line(
            (0.0, 0.0),
            (2.0 * self.frame.elapsed().as_millis() as f32, 0.0),
            4.0,
            Color::BLACK,
        );
        

        
        graphics.draw_rectangle(
            Rectangle::from_tuples(
                (20.0, 20.0),
                (20.0 + ARENA_SIZE * 5.0, 20.0 + ARENA_SIZE * 5.0),
            ),
            Color::WHITE,
        );





        let mut new_values: HashMap<(i32,i32), Vec<usize>> = HashMap::new();


        self.frame = Instant::now();

        let mut boid = Boid::new();


        // create slice of self.boids

        let boids_slice:&mut [Boid] = self.boids.as_mut_slice();

        // graphics.draw_rectangle(Rectangle::from_tuples((0.0,0.0), bottom_right), color)
        
        for i in 0..BOID_COUNT {

            #[cfg(not(debug_assertions))]{assert!(i < boids_slice.len());}

            boid.clone_from(&boids_slice[i]);
            // let mut boid = self.boids[i].clone();
            let mut v:F32x2= boid.velocity.into();
            v *= INERTIA;

            // vx = boid.velocity.0 as f32 * INERTIA;
            // vy = boid.velocity.1 as f32 * INERTIA;




            // get pixel color at current position from texture (an array of &[u8])

            let mut deflect = F32x2{x:0.0,y:0.0};
            let mut nearby = vec![];
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
                assert!(self.points.contains_key(&(
                    ((boid.position.x/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.0,
                    ((boid.position.y/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.1,
                )));
                nearby.extend_from_slice(
                    self.points.get(&(
                    ((boid.position.x/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.0,
                    ((boid.position.y/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.1,
                    )).unwrap().as_slice()
            );
                
                nearby.truncate(50);
                
                
            }
            let mut average = F32x2{x:0.0,y:0.0};
            // let average = nearby.iter().map(|x|boids_slice[*x].position).sum();
            
            for k in &nearby {
                #[cfg(not(debug_assertions))]{assert!(*k < boids_slice.len())};
                average += boids_slice[*k].position;
            }
            average *= 1.0/nearby.len() as f32;
            


            for j in nearby.iter().take(GROUP_SIZE as usize) {

                let other = &boids_slice[*j];
                let dx = (other.position.x - boid.position.x).abs();
                let dy = (other.position.y - boid.position.y).abs();
                if dx < GROUP_SPACING*2.0 && dy < GROUP_SPACING*2.0 {
                    deflect += other.velocity.into()
                }
            }

            v += (VEC_GROUP_SPACING-(average - boid.position)) * BUNCH_UP;
            // v.y +=  (GROUP_SPACING-(average.x - boid.position.x)) * BUNCH_UP;
            
            // vx -= (20.0-(average.0 - boid.position.x)) * FOLLOW;
            // vy -= (20.0-(average.1 - boid.position.y)) * FOLLOW;
            
            
            v += (VEC_ARENA_SIZE*0.5 - boid.position) * CENTER_ATTRACTION;
            // v.y += (ARENA_SIZE/2.0 - boid.position.y) * CENTER_ATTRACTION;


            v += deflect * GROUP_MOVEMENT;
            // v.y += deflect.y as f32 * GROUP_MOVEMENT;
            
            
            if nearby.len() > GROUP_SIZE as usize {
                v.x += self.rand.gen_i8() as f32 * RANDOMNESS*10.0;
                v.y += self.rand.gen_i8() as f32 * RANDOMNESS*10.0;
            }

            

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

            if new_values.contains_key(&(here.x,here.y)) {
                new_values.get_mut(&(here.x,here.y)).unwrap().push(i);
            } else {
                new_values.insert((here.x,here.y), vec![i]);
            }

            // println!("{:?}",v);

            if boid.current_point != here {
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
}
