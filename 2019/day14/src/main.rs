use std::collections::HashMap;
use topological_sort::TopologicalSort;

#[derive(Debug)]
struct Reaction {
    num_result: u64,
    result: String,
    ingredients: Vec<(u64, String)>,
}

// Parse a string in the form of "### AAAAAA"
fn parse_quantity(input: &str) -> (u64, String) {
    let split: Vec<&str> = input.split(' ').collect();
    assert!(
        split.len() == 2,
        "input: '{:?}' actual splits: {:?}",
        input,
        split
    );
    (split[0].parse::<u64>().unwrap(), split[1].to_owned())
}

fn parse_recipes(input: &str) -> HashMap<String, Reaction> {
    let mut ret: HashMap<String, Reaction> = HashMap::new();
    for line in input.split('\n') {
        let ingredients_and_result: Vec<&str> = line.split(" => ").collect();
        assert!(ingredients_and_result.len() == 2);
        let result = parse_quantity(ingredients_and_result[1]);
        let ingredients = ingredients_and_result[0]
            .split(", ")
            .map(parse_quantity)
            .collect();
        ret.insert(
            result.1.to_owned(),
            Reaction {
                num_result: result.0,
                result: result.1.to_owned(),
                ingredients,
            },
        );
    }
    ret
}

fn find_min_ore(reactions: &HashMap<String, Reaction>) -> u64 {
    let mut ts = TopologicalSort::<&str>::new();
    for reaction in reactions.values() {
        for (_, reagent) in &reaction.ingredients {
            ts.add_dependency(reaction.result.as_str(), reagent.as_str());
        }
    }

    let mut quantities: HashMap<String, u64> = HashMap::new();
    quantities.insert("FUEL".to_owned(), 1);

    while let Some(chem) = ts.pop() {
        if chem == "ORE" {
            break;
        }
        if let Some(chem_quant) = quantities.remove(chem) {
            let reaction = reactions.get(chem).unwrap();
            let mut reaction_amount = reaction.num_result;
            while reaction_amount < chem_quant {
                reaction_amount += reaction.num_result;
            }
            for (num, reagent) in &reaction.ingredients {
                let num = reaction_amount / reaction.num_result * num;
                let num = num + quantities.get(reagent).unwrap_or(&0);
                quantities.insert(reagent.to_owned(), num);
            }
        }
    }

    assert!(ts.is_empty(), "ts: {:?}", ts);

    *quantities.get("ORE").unwrap()
}

fn main() {
    let recipe = parse_recipes(include_str!("input.txt"));
    let min = find_min_ore(&recipe);
    println!("Min ore: {:?}", min);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_min_ore_1() {
        let recipe = parse_recipes(
            r#"10 ORE => 10 A
1 ORE => 1 B
7 A, 1 B => 1 C
7 A, 1 C => 1 D
7 A, 1 D => 1 E
7 A, 1 E => 1 FUEL"#,
        );
        let min = find_min_ore(&recipe);
        assert_eq!(31, min);
    }

    #[test]
    fn test_min_ore_2() {
        let recipe = parse_recipes(
            r#"9 ORE => 2 A
8 ORE => 3 B
7 ORE => 5 C
3 A, 4 B => 1 AB
5 B, 7 C => 1 BC
4 C, 1 A => 1 CA
2 AB, 3 BC, 4 CA => 1 FUEL"#,
        );
        let min = find_min_ore(&recipe);
        assert_eq!(165, min);
    }

    #[test]
    fn test_min_ore_3() {
        let recipe = parse_recipes(
            r#"157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT"#,
        );
        let min = find_min_ore(&recipe);
        assert_eq!(13312, min);
    }

    #[test]
    fn test_min_ore_4() {
        let recipe = parse_recipes(
            r#"2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF"#,
        );
        let min = find_min_ore(&recipe);
        assert_eq!(180697, min);
    }

    #[test]
    fn test_min_ore_5() {
        let recipe = parse_recipes(
            r#"171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX"#,
        );
        let min = find_min_ore(&recipe);
        assert_eq!(2210736, min);
    }
}
