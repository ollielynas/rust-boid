use core::slice;
use std::collections::HashMap;
use std::time::Instant;

// extern crate fxhash;
use fxhash::FxHashMap;
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
const BOID_COUNT: usize = 10000;



const ARENA_SIZE: f32 = 100.0;

const BUNCH_UP: f32 = 0.3;

const SPEED:f32= 0.3;

const BOID_SIZE : f32 = 1.0;

const INERTIA: f32 = 25.1;

const POINT_COUNT: usize = 100;

const RANDOMNESS: f32 = 40.1;

const GROUP_SIZE: i32 = 15;

const GROUP_SPACING: f32 = 5.1;

const GROUP_MOVEMENT: f32 = 3.8;

const CENTER_ATTRACTION: f32 = 1.1;



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
    position: (f32, f32),
    velocity: (i8, i8),
    current_point: (i32,i32)
}

impl Boid {
    fn new() -> Boid {
        Boid {
            position: (ARENA_SIZE/2.0, ARENA_SIZE/2.0),
            velocity: (0, 0),
            current_point: (-1,-1),
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

        let mut vx;
        let mut vy;

        // create slice of self.boids

        let boids_slice:&mut [Boid] = self.boids.as_mut_slice();

        // graphics.draw_rectangle(Rectangle::from_tuples((0.0,0.0), bottom_right), color)
        
        for i in 0..BOID_COUNT {

            #[cfg(not(debug_assertions))]{assert!(i < boids_slice.len());}

            boid.clone_from(&boids_slice[i]);
            // let mut boid = self.boids[i].clone();

            vx = boid.velocity.0 as f32 * INERTIA;
            vy = boid.velocity.1 as f32 * INERTIA;



            // get pixel color at current position from texture (an array of &[u8])

            let mut deflect = (0.0,0.0);
            let mut nearby = vec![];
            let here = (
                    ((boid.position.0/ARENA_SIZE)*POINT_COUNT as f32)as i32,
                    ((boid.position.1/ARENA_SIZE)*POINT_COUNT as f32)as i32,
                );
            
            for i in [(0_i32,0_i32), (-1_i32,0_i32), (1_i32,0_i32), (0_i32,1_i32), (0_i32,-1_i32)] {
                if !self.points.contains_key(&(
                    ((boid.position.0/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.0,
                    ((boid.position.1/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.1,
                )) {
                    continue;
                }
                assert!(self.points.contains_key(&(
                    ((boid.position.0/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.0,
                    ((boid.position.1/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.1,
                )));
                nearby.extend_from_slice(
                    self.points.get(&(
                    ((boid.position.0/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.0,
                    ((boid.position.1/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.1,
                    )).unwrap().as_slice()
            );
                
                nearby.truncate(50);
                
                
            }
            
            let mut average = (0.0,0.0);
            
            for k in &nearby {
                #[cfg(not(debug_assertions))]{assert!(*k < boids_slice.len())};
                average.0 +=boids_slice[*k].position.0 as f32;
                average.1 += boids_slice[*k].position.1 as f32;
            }
            average.0 /= nearby.len() as f32;
            average.1 /= nearby.len() as f32;

            


            for j in nearby.iter().take(GROUP_SIZE as usize) {

                let other = &boids_slice[*j];
                let dx = (other.position.0 - boid.position.0).abs();
                let dy = (other.position.1 - boid.position.1).abs();
                if dx < GROUP_SPACING*2.0 && dy < GROUP_SPACING*2.0 {

                    deflect.0 += other.velocity.0 as f32;
                    deflect.1 += other.velocity.1 as f32;
                }
            }

            vx += (GROUP_SPACING-(average.0 - boid.position.0)) * BUNCH_UP;
            vy +=  (GROUP_SPACING-(average.0 - boid.position.0)) * BUNCH_UP;
            
            // vx -= (20.0-(average.0 - boid.position.0)) * FOLLOW;
            // vy -= (20.0-(average.1 - boid.position.1)) * FOLLOW;
            
            
            vx += (ARENA_SIZE/2.0 - boid.position.0) * CENTER_ATTRACTION;
            vy += (ARENA_SIZE/2.0 - boid.position.1) * CENTER_ATTRACTION;


            vx += deflect.0 as f32 * GROUP_MOVEMENT;
            vy += deflect.1 as f32 * GROUP_MOVEMENT;
            
            
            if nearby.len() > GROUP_SIZE as usize {
                vx += self.rand.gen_i8() as f32 * RANDOMNESS*10.0;
                vy += self.rand.gen_i8() as f32 * RANDOMNESS*10.0;
            }

            

            vx += (self.rand.gen_i8() / 10) as f32 * RANDOMNESS;
            vy += (self.rand.gen_i8() / 10) as f32 * RANDOMNESS;

            

            let velocity_length = (vx.powi(2) + vy.powi(2)).sqrt();
            let (vx,vy) = (vx / velocity_length, vy / velocity_length);
            boid.velocity.0 = (vx * 100.0) as i8;
            boid.velocity.1 = (vy * 100.0) as i8;

            boid.position.0 += (boid.velocity.0 as f32 / 100.0)*SPEED;
            boid.position.1 += (boid.velocity.1 as f32 / 100.0)*SPEED;
            boid.position.0 = boid.position.0.min(ARENA_SIZE).max(0.0);
            boid.position.1 = boid.position.1.min(ARENA_SIZE).max(0.0);

            if boid.position.0 == ARENA_SIZE || boid.position.0 == 0.0 {
                boid.velocity.0 *= -1;
            }
            if boid.position.1 == ARENA_SIZE || boid.position.1 == 0.0 {
                boid.velocity.1 *= -1;
            }

            if new_values.contains_key(&here) {
                new_values.get_mut(&here).unwrap().push(i);
            } else {
                new_values.insert(here, vec![i]);
            }

            if boid.current_point != here {
                if let Some(list) = self.points.get_mut(&boid.current_point) {
                    list.retain(|x| x!=&i);
                }
                if let Some(new) = self.points.get_mut(&here) {
                    new.push(i);
                    boid.current_point = here;
                }else {
                    self.points.insert(here, vec![i]);
                    boid.current_point = here;
                }
            }

            boids_slice[i] = boid;
            
            
            
            graphics.draw_rectangle(Rectangle::from_tuples(((boid.position.0)*5.0+20.0-BOID_SIZE/2.0,(boid.position.1)*5.0+20.0-BOID_SIZE/2.0), ((boid.position.0)*5.0+20.0+BOID_SIZE/2.0,(boid.position.1)*5.0+20.0+BOID_SIZE/2.0)), Color::BLUE);

        }


        
        // self.points.clear();

        // mem::swap(&mut self.points, &mut new_values);
            

        

        helper.request_redraw();
    }
}
