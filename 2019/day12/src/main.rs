use std::ops;

#[derive(Clone, Copy)]
struct MoonVec {
    x: i64,
    y: i64,
    z: i64,
}

impl ops::Add<MoonVec> for MoonVec {
    type Output = MoonVec;

    fn add(self, rhs: Self) -> Self {
        MoonVec {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

struct MoonState {
    pos: MoonVec,
    vel: MoonVec,
}

struct JupiterState {
    moons: Vec<MoonState>,
}

impl JupiterState {
    fn calculate_velocity<F>(
        &self,
        current_moon: &MoonState,
        current_moon_ndx: usize,
        dimension_fn: F,
    ) -> i64
    where
        F: Fn(MoonVec) -> i64,
    {
        self.moons
            .iter()
            .enumerate()
            .filter(|&(ndx, _)| ndx != current_moon_ndx)
            .map(|(_, moon)| 0)
            .sum()
    }

    fn step(&self) -> Self {
        // calculate velocity
        let moons = self
            .moons
            .iter()
            .enumerate()
            .map(|(ndx, moon)| MoonState {
                pos: moon.pos,
                vel: moon.vel
                    + MoonVec {
                        x: self.calculate_velocity(ndx, |v| v.x),
                        y: self.calculate_velocity(ndx, |v| v.y),
                        z: self.calculate_velocity(ndx, |v| v.z),
                    },
            })
            .collect();
        JupiterState { moons }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
}
