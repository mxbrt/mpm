use glam::{vec2, Mat2, Vec2};

// MLS-MPM fluid dynamics
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
    pub pos: Vec2,
    pub vel: Vec2,
    c: Mat2,
    mass: f32,
}

#[derive(Clone)]
struct Cell {
    vel: Vec2,
    mass: f32,
}

pub struct Simulator {
    pub particles: Vec<Particle>,
    grid: Vec<Cell>,
}

impl Simulator {
    pub fn new(particle_positions: &Vec<Vec2>) -> Simulator {
        let particles = particle_positions
            .iter()
            .map(|pos| Particle {
                pos: pos.clone(),
                vel: Vec2::ZERO,
                c: Mat2::ZERO,
                mass: 1.0,
            })
            .collect();
        let grid = vec![
            Cell {
                vel: Vec2::ZERO,
                mass: 0.0
            };
            NUM_CELLS
        ];

        Simulator { particles, grid }
    }

    pub fn waterbox() -> Simulator {
        let mut particles = Vec::<Vec2>::new();
        let border = (GRID_RES as f32 * 0.2) as usize;
        for i in border..GRID_RES - border {
            for j in border..GRID_RES - border {
                particles.push(vec2(i as f32, j as f32));
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
            cell.vel = Vec2::ZERO;
        }
    }

    fn particles_to_grid_1(&mut self) {
        let mut weights = [Vec2::ZERO; 3];
        for p in &self.particles {
            let grid_pos = p.pos.floor();
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
        let mut weights = [Vec2::ZERO; 3];
        for p in &self.particles {
            let grid_pos = p.pos.floor();
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

            let mut stress = Mat2::from_cols_array(&[-pressure, 0.0, 0.0, -pressure]);
            let trace = p.c.col(1).x + p.c.col(0).y;
            let strain = Mat2::from_cols_array(&[p.c.col(0).x, trace, trace, p.c.col(1).y]);

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
                cell.vel += DT * vec2(0.0, GRAVITY);

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
        let mut weights = [Vec2::ZERO; 3];
        for p in &mut self.particles {
            p.vel = Vec2::ZERO;
            let grid_pos = p.pos.floor();
            init_weights(&p, &grid_pos, &mut weights);

            let mut b = Mat2::ZERO;
            for x in 0..3 {
                for y in 0..3 {
                    let weight = weights[x].x * weights[y].y;
                    let (cell_dist, cell_idx) = init_cell(&p.pos, &grid_pos, x, y);
                    let weighted_velocity = self.grid[cell_idx].vel * weight;

                    let velocity_term = {
                        let row0 = weighted_velocity * cell_dist.x;
                        let row1 = weighted_velocity * cell_dist.y;
                        Mat2::from_cols_array(&[row0.x, row0.y, row1.x, row1.y])
                    };
                    b += velocity_term;
                    p.vel += weighted_velocity;
                }
            }

            p.c = b * 4.0;
            p.pos += p.vel * DT;

            let grid_boundary = GRID_RES as f32 - 2.0;
            p.pos = p
                .pos
                .clamp(vec2(1.0, 1.0), vec2(grid_boundary, grid_boundary));
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

    pub fn render(&self, img: &mut Vec<u8>, width: usize, height: usize) {
        for i in 0..img.len() {
            img[i] = 0;
        }
        for p in &self.particles {
            let pixel_x = (p.pos.x * (width as f32 / GRID_RES as f32)) as usize;
            let pixel_y = height - (p.pos.y * (height as f32 / GRID_RES as f32)) as usize;
            let pixel_idx = (pixel_y * width + pixel_x) * 4;
            img[pixel_idx] = (p.mass * 255.0) as u8;
            img[pixel_idx + 1] = (p.vel.x * 255.0) as u8;
            img[pixel_idx + 2] = (p.vel.y * 255.0) as u8;
            img[pixel_idx + 3] = 255;
        }
    }
}

fn init_weights(p: &Particle, grid_pos: &Vec2, weights: &mut [Vec2; 3]) {
    let half = vec2(0.5, 0.5);
    let cell_diff = (p.pos - *grid_pos) - half;
    weights[0] = vec2(0.5, 0.5) * (half - cell_diff).powf(2.0);
    weights[1] = vec2(0.75, 0.75) - cell_diff.powf(2.0);
    weights[2] = vec2(0.5, 0.5) * (half + cell_diff).powf(2.0);
}

fn init_cell(particle_pos: &Vec2, grid_pos: &Vec2, x: usize, y: usize) -> (Vec2, usize) {
    let half = vec2(0.5, 0.5);
    let cell_pos = vec2(grid_pos.x + x as f32 - 1.0, grid_pos.y + y as f32 - 1.0);
    let cell_dist = (cell_pos - *particle_pos) + half;
    let cell_idx = (grid_pos.x as usize + x - 1) * GRID_RES + (grid_pos.y as usize + y - 1);
    (cell_dist, cell_idx)
}
