use std::error::Error;

fn calculate_fuel_required(mass: u32) -> u32 {
    if mass < 6 {
        0
    } else {
        mass / 3 - 2
    }
}

fn calculate_fuel_required_including_fuel(mass: u32) -> u32 {
    if mass == 0 {
        0
    } else {
        let fuel_required = calculate_fuel_required(mass);
        fuel_required + calculate_fuel_required_including_fuel(fuel_required)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut naive_total = 0;
    let mut total = 0;
    for line in std::fs::read_to_string("input.txt")?.lines() {
        let mass = u32::from_str_radix(line, 10)?;
        naive_total += calculate_fuel_required(mass);
        total += calculate_fuel_required_including_fuel(mass);
    }

    println!("Total fuel needed: {}", naive_total);
    println!("Total fuel needed (including fuel): {}", total);
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
        assert_eq!(2, calculate_fuel_required_including_fuel(14));
        assert_eq!(966, calculate_fuel_required_including_fuel(1969));
        assert_eq!(50346, calculate_fuel_required_including_fuel(100756));
    }
}
