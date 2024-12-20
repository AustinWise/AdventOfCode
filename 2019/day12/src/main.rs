use std::{cmp::Ordering, ops};

#[derive(Debug, Clone, Copy, PartialEq)]
struct MoonVec {
    x: i64,
    y: i64,
    z: i64,
}

impl MoonVec {
    fn sum_abs_values(&self) -> i64 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
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

#[derive(Debug, PartialEq)]
struct MoonState {
    pos: MoonVec,
    vel: MoonVec,
}

impl MoonState {
    fn new(x: i64, y: i64, z: i64) -> Self {
        MoonState {
            pos: MoonVec { x, y, z },
            vel: MoonVec { x: 0, y: 0, z: 0 },
        }
    }

    #[allow(dead_code)]
    fn new_with_vel(x: i64, y: i64, z: i64, vel_x: i64, vel_y: i64, vel_z: i64) -> Self {
        MoonState {
            pos: MoonVec { x, y, z },
            vel: MoonVec {
                x: vel_x,
                y: vel_y,
                z: vel_z,
            },
        }
    }

    fn energy(&self) -> i64 {
        let potential_energy = self.pos.sum_abs_values();
        let kinetic_energy = self.vel.sum_abs_values();
        potential_energy * kinetic_energy
    }
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
        F: Fn(&MoonVec) -> i64,
    {
        self.moons
            .iter()
            .enumerate()
            .filter(|&(ndx, _)| ndx != current_moon_ndx)
            .map(
                |(_, moon)| match dimension_fn(&moon.pos).cmp(&dimension_fn(&current_moon.pos)) {
                    Ordering::Less => -1,
                    Ordering::Equal => 0,
                    Ordering::Greater => 1,
                },
            )
            .sum()
    }

    fn step(&self) -> Self {
        // calculate velocity
        let moons: Vec<MoonState> = self
            .moons
            .iter()
            .enumerate()
            .map(|(ndx, moon)| MoonState {
                pos: moon.pos,
                vel: moon.vel
                    + MoonVec {
                        x: self.calculate_velocity(moon, ndx, |v| v.x),
                        y: self.calculate_velocity(moon, ndx, |v| v.y),
                        z: self.calculate_velocity(moon, ndx, |v| v.z),
                    },
            })
            .collect();
        let moons: Vec<MoonState> = moons
            .iter()
            .map(|moon| MoonState {
                pos: moon.pos + moon.vel,
                vel: moon.vel,
            })
            .collect();
        JupiterState { moons }
    }

    fn total_energy(&self) -> i64 {
        self.moons.iter().map(|moon| moon.energy()).sum()
    }
}

fn main() {
    let mut state = JupiterState {
        moons: vec![
            MoonState::new(-16, 15, -9),
            MoonState::new(-14, 5, 4),
            MoonState::new(2, 0, 6),
            MoonState::new(-3, 18, 9),
        ],
    };

    for _ in 0..1000 {
        state = state.step();
    }

    println!("part 1: {}", state.total_energy());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_sample_data() -> JupiterState {
        JupiterState {
            moons: vec![
                MoonState::new(-1, 0, 2),
                MoonState::new(2, -10, -7),
                MoonState::new(4, -8, 8),
                MoonState::new(3, 5, -1),
            ],
        }
    }

    #[test]
    fn test_step() {
        let state = get_sample_data();

        let state = state.step();

        assert_eq!(
            state.moons,
            vec![
                MoonState::new_with_vel(2, -1, 1, 3, -1, -1),
                MoonState::new_with_vel(3, -7, -4, 1, 3, 3),
                MoonState::new_with_vel(1, -7, 5, -3, 1, -3),
                MoonState::new_with_vel(2, 2, 0, -1, -3, 1),
            ]
        );

        let state = state.step();

        assert_eq!(
            state.moons,
            vec![
                MoonState::new_with_vel(5, -3, -1, 3, -2, -2),
                MoonState::new_with_vel(1, -2, 2, -2, 5, 6),
                MoonState::new_with_vel(1, -4, -1, 0, 3, -6),
                MoonState::new_with_vel(1, -4, 2, -1, -6, 2),
            ]
        )
    }

    #[test]
    fn test_engery() {
        let mut state = get_sample_data();
        for _ in 0..10 {
            state = state.step();
        }
        assert_eq!(179, state.total_energy());
    }
}
