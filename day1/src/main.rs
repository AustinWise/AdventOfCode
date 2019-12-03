use std::error::Error;

fn calculate_fuel_required(mass: u32) -> u32 {
    if mass < 6 {
        0
    } else {
        mass / 3 - 2
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut total = 0;
    for line in std::fs::read_to_string("input.txt")?.lines() {
        let mass = u32::from_str_radix(line, 10)?;
        total += calculate_fuel_required(mass);
    }

    println!("Total fuel needed: {}", total);
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_input() {
        assert_eq!(0, calculate_fuel_required(0));
        assert_eq!(2, calculate_fuel_required(12));
        assert_eq!(2, calculate_fuel_required(14));
        assert_eq!(654, calculate_fuel_required(1969));
        assert_eq!(33583, calculate_fuel_required(100756));
    }
}
