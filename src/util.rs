use std::f32::consts::PI;

use cgmath::Vector3;
use rand::prelude::*;

pub fn rand_betw
<
    T: std::cmp::PartialOrd +
    rand::distributions::uniform::SampleUniform,
>
(
    n1: T, 
    n2: T
) -> T {
    let mut r = thread_rng();
    r.gen_range(n1..n2)
}

pub struct SecondOrderDynamics { // make it so that input x yields in a smooth, natural output y
    xp: Vector3<f32>, // previous inputs
    y: Vector3<f32>, 
    yd: Vector3<f32>,

    //constants
    k1: f32,
    k2: f32, 
    k3: f32,

    t_critical: f32,
}

impl SecondOrderDynamics {
    pub fn new(f: f32, z: f32, r: f32, x0: Vector3<f32>) -> Self {
        let k1 = z / (PI * f);
        let k2 = 1.0 / ((2.0 * PI * f) * (2.0 * PI * f));
        let k3 = r * z / (2.0 * PI * f);
        
        let xp = x0;
        let y = x0;
        let yd = cgmath::vec3(0.0, 0.0, 0.0);

        // critical timestep threshold where the simulation would
        // become unstable past it

        let t_critical = (f32::sqrt(4.0*k2 + k1 * k1) - k1) * 0.8; // multiply by an arbitrary value to be safe

        Self {
            k1,
            k2,
            k3,

            xp,
            y,
            yd,

            t_critical,
        }
    }

    pub fn update(&mut self, mut timestep: f32, x: Vector3<f32>) -> Vector3<f32> {
        let xd = (x - self.xp) / timestep;
        self.xp = x;

        let iterations = f32::ceil(timestep / self.t_critical); // take extra iterations if t > tcrit
        timestep = timestep / iterations; // lower timesteps

        for i in 0..iterations as usize {
            self.y = self.y + timestep * self.yd;
            self.yd = self.yd + timestep * (x + self.k3*xd - self.y - self.k1*self.yd) / self.k2;
        }

        self.y
    }
}
