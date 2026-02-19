use macroquad::color;
use macroquad::prelude::*;


#[derive(Clone)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    radius: f32,
    color: Color,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            vel: Vec2::default(),
            radius: 10.0,
            color: WHITE,
        }
    }
}

impl Particle {
    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, self.radius, self.color);
    }
}



#[derive(Default)]
struct GridCell<'a> {
    /// upper left, right, bottom left, right
    children: Option<[Box<GridCell<'a>>; 4]>,

    particles: Vec<&'a Particle>,

    pos1: Vec2, // upper left corner
    pos2: Vec2, // bottom right corner
}

impl<'a> GridCell<'a> {
    fn new(pos1: Vec2, pos2: Vec2, part: usize, particles: Vec<&'a Particle>) -> Self {
        let mut cell = GridCell::default();

        let mut p1 = pos1;
        let mut p2 = pos2;
        let size = pos2 - pos1;

        if part == 2 || part == 4 {
            p1.x += size.x / 2.0;
        } else {
            p2.x -= size.x / 2.0;
        }

        if part == 3 || part == 4 {
            p1.y += size.y / 2.0;
        } else {
            p2.y -= size.y / 2.0;
        }

        cell.pos1 = p1;
        cell.pos2 = p2;

        for particle in particles {
            if particle.pos.x > p1.x && particle.pos.x < p2.x &&
                particle.pos.y > p1.y && particle.pos.y < p2.y {
                cell.particles.push(particle);
            }
        }

        if cell.particles.len() > 6 {
            let mut child_cells: [Box<GridCell>; 4] = Default::default();

            for i in 0..4 {
                child_cells[i] = Box::new(GridCell::new(cell.pos1, cell.pos2, i+1, Vec::new()));

            }
            cell.children = Some(child_cells);
        }

        cell
    }
}

impl<'a> GridCell<'a> {
    fn draw(&self) {
        if let Some(children) = self.children.as_ref() {
            for child in children.iter() {
                child.draw();
            }
        } else {
            draw_rectangle(self.pos1.x, self.pos1.y, self.pos1.x, self.pos1.y, WHITE);
        }
    }
}


#[derive(Default)]
struct State<'a> {
    particles: Vec<Particle>,
    grid: GridCell<'a>,
    gravity: Vec2,
    friction: f32,
    last_iter_count: usize,
}


impl<'a> State<'a> {
    fn draw(&self) {
        for particle in &self.particles {
            particle.draw();
        }

        self.grid.draw()
    }

    fn update(&mut self, dt: f32) {
        let screen_width =  screen_width();
        let screen_height =  screen_height();


        // Mouse dragging
        if is_mouse_button_down(MouseButton::Left) {
            // Find selected particle
            let mouse_pos = Vec2::new(mouse_position().0, mouse_position().1);
            for particle in &mut self.particles {
                if (mouse_pos - particle.pos).length() < particle.radius {
                    particle.vel = -mouse_delta_position() * Vec2::new(screen_width, screen_height) / dt * 0.5;
                    particle.pos = mouse_pos;
                }
            }

        }

        let desired_fps = 60;
        let mut iter_count = 1;

        if self.last_iter_count != 0 {
            // Time each iteration took last frame
            let desired_dt = 1.0 / desired_fps as f32;
            iter_count = ((self.last_iter_count as f32 * desired_dt / dt) as usize).max(1);
        }

        self.last_iter_count = iter_count;

        for iter in 0..iter_count {
            let dt = dt / iter_count as f32;

            for i in 0..self.particles.len() {
                // First we update ourselves
                {
                    let p1 = &mut self.particles[i];
                    let elasticity = 0.8;

                    p1.vel += self.gravity * dt;
                    p1.pos += p1.vel * dt;

                    // Right wall
                    if p1.pos.x + p1.radius > screen_width {
                        p1.pos.x = screen_width - p1.radius;
                        if p1.vel.x > 0.0 {
                            p1.vel.x *= -elasticity;
                            p1.vel.y *= 1.0 - self.friction;
                        }
                    }

                    // Left wall
                    if p1.pos.x - p1.radius < 0.0 {
                        p1.pos.x = p1.radius;
                        if p1.vel.x < 0.0 {
                            p1.vel.x *= -elasticity;
                            p1.vel.y *= 1.0 - self.friction;
                        }
                    }

                    // Bottom wall
                    if p1.pos.y + p1.radius > screen_height {
                        p1.pos.y = screen_height - p1.radius;
                        if p1.vel.y > 0.0 {
                            p1.vel.y *= -elasticity;
                            p1.vel.x *= 1.0 - self.friction;
                        }
                    }

                    // Top wall
                    if p1.pos.y - p1.radius < 0.0 {
                        p1.pos.y = p1.radius;
                        if p1.vel.y < 0.0 {
                            p1.vel.y *= -elasticity;
                            p1.vel.x *= 1.0 - self.friction;
                        }
                    }
                }

                if i == self.particles.len() - 1 { continue; }


                // Now check against other particles
                for j in i+1..self.particles.len() {
                    let (left, right) = self.particles.split_at_mut(j);
                    let p1 = &mut left[i];
                    let p2 = &mut right[0];

                    let dist = (p1.pos - p2.pos).length();

                    // Continue if we dont collide
                    let overlap = p1.radius + p2.radius - dist;

                    if overlap < 0.0 { continue; }
                    if dist < 0.01 { continue; }

                    let rel_vel = (p2.vel - p1.vel) * 0.5;

                    let delta_pos = p2.pos - p1.pos;
                    let normal = delta_pos.normalize();

                    // Push them out of each other
                    p1.pos -= normal * overlap * 0.5;
                    p2.pos += normal * overlap * 0.5;

                    let vel_along_normal = normal * normal.dot(rel_vel);
                    let vel_along_tangent = rel_vel - vel_along_normal;

                    p1.vel += rel_vel;
                    p2.vel -= rel_vel;

                    let elasticity = 0.8;
                    p1.vel += vel_along_normal * elasticity;
                    p2.vel -= vel_along_normal * elasticity;

                    p1.vel -= vel_along_tangent * (1.0 - self.friction);
                    p2.vel += vel_along_tangent * (1.0 - self.friction);
                }
            }
        }
    }

    fn get_neighbors(&'a self, cell: &GridCell<'a>) -> Vec<&'a GridCell<'a>>
    {
        let mut neighbors: Vec<&GridCell> = Vec::default();
        let leaf_cells = get_leaf_cells(&self.grid);

        let p1 = cell.pos1;
        let p2 = cell.pos2;

        for leaf in leaf_cells {
            if leaf.pos1 == p1 && leaf.pos2 == p2 { continue; }

            if leaf.pos1.x <= p2.x &&
                leaf.pos2.x >= p1.x &&
                leaf.pos1.y <= p2.y &&
                leaf.pos2.y >= p1.y {
                neighbors.push(leaf);
            }
        }

        neighbors
    }


}


fn get_leaf_cells<'a>(cell: &'a GridCell) -> Vec<&'a GridCell<'a>> {
    let mut cells: Vec<&GridCell> = Vec::new();

    if cell.children.is_some() {
        for child in cell.children.as_ref().unwrap() {
            cells.append(&mut get_leaf_cells(child));
        }
    } else {
        cells.push(cell);
    }

    cells
}

#[macroquad::main("Hello World!")]
async fn main() {
    println!("Hello, world!");

    let mut state = State::default();
    state.gravity = Vec2::new(10.0, 400.0);

    for i in 0..500 {
        state.particles.push(Particle {
            pos: Vec2::new(100.0 + (i % 100) as f32, 100.0 + (i / 100) as f32),
            vel: Vec2::default(),
            radius: 10.0,
            color: WHITE,
        });
    }


    loop {
        clear_background(DARKGRAY);

        state.draw();
        state.update(get_frame_time());

        draw_text(&format!("FPS: {}", get_fps()), 20.0, 30.0, 30.0, WHITE);
        draw_text(&format!("Iter count: {}", state.last_iter_count), 18.0, 50.0, 30.0, WHITE);

        next_frame().await
    }
}
