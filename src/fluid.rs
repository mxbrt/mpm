// MLS-MPM fluid dynamics
extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

pub static GRID_RES: usize = 256;
static NUM_CELLS: usize = GRID_RES * GRID_RES;

// simulation
static DT: f32 = 0.2;
static GRAVITY: f32 = -0.3;

// fluid parameters
static REST_DENSITY: f32 = 4.0;
static DYNAMIC_VISCOSITY: f32 = 0.1;

// equation of state
static EOS_STIFFNESS: f32 = 10.0;
static EOS_POWER: f32 = 4.0;

#[derive(Debug)]
pub struct Particle {
    pub pos: glm::Vec2,
    pub vel: glm::Vec2,
    c: glm::Mat2,
    mass: f32,
}

#[derive(Clone)]
struct Cell {
    vel: glm::Vec2,
    mass: f32,
}

pub struct Simulator {
    pub particles: Vec<Particle>,
    grid: Vec<Cell>,
}

impl Simulator {
    pub fn new(particle_positions: &Vec<glm::Vec2>) -> Simulator {
        let particles = particle_positions
            .iter()
            .map(|pos| Particle {
                pos: pos.clone(),
                vel: glm::Vec2::zeros(),
                c: glm::Mat2::zeros(),
                mass: 1.0,
            })
            .collect();
        let grid = vec![
            Cell {
                vel: glm::Vec2::zeros(),
                mass: 0.0
            };
            NUM_CELLS
        ];

        Simulator { particles, grid }
    }

    pub fn waterbox() -> Simulator {
        let mut particles = Vec::<glm::Vec2>::new();
        let border = (GRID_RES as f32 * 0.2) as usize;
        for i in border..GRID_RES - border {
            for j in border..GRID_RES - border {
                particles.push(glm::vec2(i as f32, j as f32));
            }
        }
        Simulator::new(&particles)
    }

    pub fn print(&self) {
        for p in &self.particles {
            println!("{:#?}", p.pos);
        }
    }

    pub fn step(&mut self) {
        for _ in 0..(1.0 / DT) as usize {
            self.clear_grid();
            self.particles_to_grid_1();
            self.particles_to_grid_2();
            self.update_grid();
            self.grid_to_particles();
        }
    }

    fn clear_grid(&mut self) {
        for cell in &mut self.grid {
            cell.mass = 0.0;
            cell.vel = glm::Vec2::zeros();
        }
    }

    fn particles_to_grid_1(&mut self) {
        let mut weights = [glm::Vec2::zeros(); 3];
        for p in &self.particles {
            let grid_pos = glm::floor(&p.pos);
            init_weights(&p, &grid_pos, &mut weights);

            for x in 0..3 {
                for y in 0..3 {
                    let weight = weights[x].x * weights[y].y;
                    let (cell_dist, cell_idx) = init_cell(&p.pos, &grid_pos, x, y);
                    let q = p.c * cell_dist;

                    let mass_contrib = weight * p.mass;
                    let cell = &mut self.grid[cell_idx];

                    cell.mass += mass_contrib;
                    cell.vel += mass_contrib * (p.vel + q);
                }
            }
        }
    }

    fn particles_to_grid_2(&mut self) {
        let mut weights = [glm::Vec2::zeros(); 3];
        for p in &self.particles {
            let grid_pos = glm::floor(&p.pos);
            init_weights(&p, &grid_pos, &mut weights);

            let mut density = 0.0;
            for x in 0..3 {
                for y in 0..3 {
                    let weight = weights[x].x * weights[y].y;
                    let cell_idx =
                        (grid_pos.x as usize + x - 1) * GRID_RES + (grid_pos.y as usize + y - 1);
                    density += self.grid[cell_idx].mass * weight;
                }
            }

            let volume = p.mass / density;
            let pressure =
                (EOS_STIFFNESS * (density / REST_DENSITY).powf(EOS_POWER) - 1.0).max(-0.1);

            let mut stress = glm::mat2(-pressure, 0.0, 0.0, -pressure);
            let trace = p.c.column(1).x + p.c.column(0).y;
            let strain = glm::mat2(p.c.column(0).x, trace, trace, p.c.column(1).y);

            let viscosity_term = DYNAMIC_VISCOSITY * strain;
            stress += viscosity_term;

            let eq_16_term_0 = -volume * 4.0 * stress * DT;
            for x in 0..3 {
                for y in 0..3 {
                    let weight = weights[x].x * weights[y].y;
                    let (cell_dist, cell_idx) = init_cell(&p.pos, &grid_pos, x, y);
                    let cell = &mut self.grid[cell_idx];
                    let momentum = (eq_16_term_0 * weight) * cell_dist;
                    cell.vel += momentum;
                }
            }
        }
    }

    fn update_grid(&mut self) {
        for i in 0..self.grid.len() {
            let cell = &mut self.grid[i];
            if cell.mass > 0.0 {
                cell.vel /= cell.mass;
                cell.vel += DT * glm::vec2(0.0, GRAVITY);

                // boundary conditions
                let x = i / GRID_RES;
                let y = i % GRID_RES;
                if x < 2 || x > GRID_RES - 3 {
                    cell.vel.x = 0.0;
                }
                if y < 2 || y > GRID_RES - 3 {
                    cell.vel.y = 0.0;
                }
            }
        }
    }

    fn grid_to_particles(&mut self) {
        let mut weights = [glm::Vec2::zeros(); 3];
        for p in &mut self.particles {
            p.vel = glm::Vec2::zeros();
            let grid_pos = glm::floor(&p.pos);
            init_weights(&p, &grid_pos, &mut weights);

            let mut b = glm::Mat2::zeros();
            for x in 0..3 {
                for y in 0..3 {
                    let weight = weights[x].x * weights[y].y;
                    let (cell_dist, cell_idx) = init_cell(&p.pos, &grid_pos, x, y);
                    let weighted_velocity = self.grid[cell_idx].vel * weight;

                    let velocity_term = {
                        let row0 = weighted_velocity * cell_dist.x;
                        let row1 = weighted_velocity * cell_dist.y;
                        glm::mat2(row0.x, row0.y, row1.x, row1.y)
                    };
                    b += velocity_term;
                    p.vel += weighted_velocity;
                }
            }

            p.c = b * 4.0;
            p.pos += p.vel * DT;
            p.pos = glm::clamp(&p.pos, 1.0, GRID_RES as f32 - 2.0);

            let x_n = p.pos + p.vel;
            let wall_min = 3.0;
            let wall_max = GRID_RES as f32 - 4.0;
            if x_n.x < wall_min {
                p.vel.x += wall_min - x_n.x
            }
            if x_n.x > wall_max {
                p.vel.x += wall_max - x_n.x
            }
            if x_n.y < wall_min {
                p.vel.y += wall_min - x_n.y
            }
            if x_n.y > wall_max {
                p.vel.y += wall_max - x_n.y
            }
        }
    }

    pub fn render(&self, img: &mut Vec<f32>, width: usize, height: usize) {
        for i in 0..img.len() {
            img[i] = 0.0;
        }
        for p in &self.particles {
            let pixel_x = (p.pos.x * (width as f32 / GRID_RES as f32)) as usize;
            let pixel_y = (p.pos.y * (height as f32 / GRID_RES as f32)) as usize;
            let pixel_idx = (pixel_y * width + pixel_x) * 4;
            img[pixel_idx] = p.mass;
            img[pixel_idx + 1] = p.vel.x;
            img[pixel_idx + 2] = p.vel.y;
            img[pixel_idx + 3] = 1.0;
        }
    }
}

fn init_weights(p: &Particle, grid_pos: &glm::Vec2, weights: &mut [glm::Vec2; 3]) {
    let half = glm::vec2(0.5, 0.5);
    let two = glm::vec2(2.0, 2.0);
    let cell_diff = (p.pos - grid_pos) - half;
    weights[0] = 0.5 * glm::pow(&(half - cell_diff), &two);
    weights[1] = glm::vec2(0.75, 0.75) - glm::pow(&cell_diff, &two);
    weights[2] = 0.5 * glm::pow(&(half + cell_diff), &two);
}

fn init_cell(
    particle_pos: &glm::Vec2,
    grid_pos: &glm::Vec2,
    x: usize,
    y: usize,
) -> (glm::Vec2, usize) {
    let half = glm::vec2(0.5, 0.5);
    let cell_pos = glm::vec2(grid_pos.x + x as f32 - 1.0, grid_pos.y + y as f32 - 1.0);
    let cell_dist = (cell_pos - particle_pos) + half;
    let cell_idx = (grid_pos.x as usize + x - 1) * GRID_RES + (grid_pos.y as usize + y - 1);
    (cell_dist, cell_idx)
}
