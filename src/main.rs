use std::collections::HashMap;
use std::hash::Hash;
use std::mem;
use std::time::Instant;

use array_fu::array;
use speedy2d::color::Color;

use speedy2d::shape::{Rect, Rectangle};
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::{Graphics2D, Window};
use turborand::prelude::*;

const BOID_COUNT: usize = 1000;

const ARENA_SIZE: f32 = 100.0;

const BUNCH_UP: f32 = 0.1;

const SPEED:f32= 0.4;

const BOID_SIZE : f32 = 1.0;

const INERTIA: f32 = 5.1;

const POINT_COUNT: usize = 100;

const RANDOMNESS: f32 = 20.1;

const GROUP_SIZE: i32 = 8;

const GROUP_SPACING: f32 = 10.1;

const GROUP_MOVEMENT: f32 = 2.8;



#[derive(Clone, Copy)]
struct Boid {
    position: (f32, f32),
    velocity: (i8, i8),
}

impl Boid {
    fn new() -> Boid {
        Boid {
            position: (0.0, 0.0),
            velocity: (0, 0),
        }
    }
}

fn main() {
    let rand = Rng::new();
    let window = Window::new_centered("Title", (650, 640)).unwrap();

    window.run_loop(MyWindowHandler {
        frame: Instant::now(),
        boids: vec![Boid::new(); BOID_COUNT],
        rand: rand,
        points: HashMap::new(),
    });
}

struct MyWindowHandler {
    frame: Instant,
    boids: Vec<Boid>,
    rand: Rng,
    points: HashMap<(i32,i32), Vec<usize>>,
}

impl WindowHandler for MyWindowHandler {
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        let scale = helper.get_scale_factor();
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
        // graphics.draw_rectangle(Rectangle::from_tuples((0.0,0.0), bottom_right), color)
        for i in 0..BOID_COUNT {

            #[cfg(not(debug_assertions))]{assert!(i < self.boids.len());}


            let mut boid = self.boids[i].clone();

            let mut vx = boid.velocity.0 as f32 * INERTIA;
            let mut vy = boid.velocity.1 as f32 * INERTIA;



            // get pixel color at current position from texture (an array of &[u8])

            #[cfg(not(debug_assertions))]{assert!(vector_index < POINT_COUNT*POINT_COUNT);}
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
                nearby.append(&mut self.points.get(&(
                    ((boid.position.0/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.0,
                    ((boid.position.1/ARENA_SIZE)*POINT_COUNT as f32)as i32 + i.1,
                )).unwrap().clone());
                nearby.truncate(50);
                
                
            }
            
            let mut average = (0.0,0.0);
            
            for i in &nearby {
                average.0 += self.boids[*i].position.0 as f32;
                average.1 += self.boids[*i].position.1 as f32;
            }
            average.0 /= nearby.len() as f32;
            average.1 /= nearby.len() as f32;
            for i in nearby.iter().take(GROUP_SIZE as usize) {

                let other = &self.boids[*i];
                let dx = (other.position.0 - boid.position.0).abs();
                let dy = (other.position.1 - boid.position.1).abs();
                if dx < GROUP_SPACING && dy < GROUP_SPACING {

                    deflect.0 += other.velocity.0 as f32;
                    deflect.1 += other.velocity.1 as f32;
                }
            }

            vx += (GROUP_SPACING-(average.0 - boid.position.0)) * BUNCH_UP;
            vy +=  (GROUP_SPACING-(average.0 - boid.position.0)) * BUNCH_UP;
            
            // vx -= (20.0-(average.0 - boid.position.0)) * FOLLOW;
            // vy -= (20.0-(average.1 - boid.position.1)) * FOLLOW;
            
            


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


            self.boids[i] = boid;
            
            graphics.draw_rectangle(Rectangle::from_tuples(((boid.position.0)*5.0+20.0-BOID_SIZE/2.0,(boid.position.1)*5.0+20.0-BOID_SIZE/2.0), ((boid.position.0)*5.0+20.0+BOID_SIZE/2.0,(boid.position.1)*5.0+20.0+BOID_SIZE/2.0)), Color::BLUE);



        }
        
        self.points.clear();

        mem::swap(&mut self.points, &mut new_values);
            

        

        helper.request_redraw();
    }
}
