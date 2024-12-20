use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::Lines;

#[derive(Debug)]
enum MyError {
    ParseError,
    DuplicateEntry,
    MissingLink,
    NodeNotFound,
    Loop,
}
impl Error for MyError {}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Object {
    parent: Option<usize>,
}

struct OrbitMap {
    objects: Vec<Object>,
    object_names: HashMap<String, usize>,
}

impl OrbitMap {
    fn new() -> OrbitMap {
        let mut ret = OrbitMap {
            objects: vec![Object { parent: Some(0) }],
            object_names: HashMap::new(),
        };
        ret.object_names.insert("COM".to_string(), 0);
        ret
    }

    fn get_or_add_obect(&mut self, name: &str) -> usize {
        match self.object_names.get(name) {
            Some(ndx) => *ndx,
            None => {
                self.objects.push(Object { parent: None });
                let ndx = self.objects.len() - 1;
                self.object_names.insert(name.to_string(), ndx);
                ndx
            }
        }
    }

    fn add_orbit(&mut self, orbited_name: &str, orbitor_name: &str) -> Result<(), MyError> {
        let orbited_ndx = self.get_or_add_obect(orbited_name);
        let orbitor_ndx = self.get_or_add_obect(orbitor_name);
        let orbitor = self.objects.get_mut(orbitor_ndx).unwrap();
        match orbitor.parent {
            Some(_) => Err(MyError::DuplicateEntry),
            None => {
                orbitor.parent = Some(orbited_ndx);
                Ok(())
            }
        }
    }

    fn add_lines(&mut self, lines: &mut Lines) -> Result<(), MyError> {
        for line in lines {
            let parts: Vec<&str> = line.split(')').collect();
            if parts.len() != 2 {
                return Err(MyError::ParseError);
            }
            self.add_orbit(parts[0], parts[1])?;
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), MyError> {
        for obj in &self.objects {
            if obj.parent.is_none() {
                return Err(MyError::MissingLink);
            }
        }
        Ok(())
    }

    fn total_number_of_orbits(&self) -> Result<usize, MyError> {
        fn count_one(map: &OrbitMap, obj: &Object) -> usize {
            let mut obj = obj;
            let mut ret = 1;
            while obj.parent.unwrap() != 0 {
                obj = &map.objects[obj.parent.unwrap()];
                ret += 1;
            }
            ret
        }

        self.validate()?;

        let mut ret = 0;
        for obj in self.objects.iter().skip(1) {
            //TODO: memoize
            ret += count_one(self, obj);
        }
        Ok(ret)
    }

    fn find_distance_between(&self, a_name: &str, b_name: &str) -> Result<usize, MyError> {
        self.validate()?;

        let a_ndx = match self.object_names.get(a_name) {
            Some(ndx) => *ndx,
            None => return Err(MyError::NodeNotFound),
        };
        let b_ndx = match self.object_names.get(b_name) {
            Some(ndx) => *ndx,
            None => return Err(MyError::NodeNotFound),
        };

        let a_nodes = {
            let mut count = 0;
            let mut a_nodes = HashMap::new();
            let mut ndx = a_ndx;
            while ndx != 0 {
                if a_nodes.insert(ndx, count).is_some() {
                    return Err(MyError::Loop);
                }
                let node = &self.objects[ndx];
                ndx = node.parent.unwrap();
                count += 1;
            }
            a_nodes.insert(0, count);
            a_nodes
        };

        let mut b_distance = 0;
        let mut common_parent = b_ndx;
        loop {
            if a_nodes.contains_key(&common_parent) {
                break;
            } else if common_parent == 0 {
                return Err(MyError::NodeNotFound);
            } else {
                common_parent = self.objects[common_parent].parent.unwrap();
                b_distance += 1;
            }
        }

        //The above code calculates the number of edges from each node to the common parent node.
        //Since we want to not count moving from ourselves to the orbited planet, substrate one
        //for each starting node.
        Ok(b_distance + a_nodes[&common_parent] - 2)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut oribit_map = OrbitMap::new();
    oribit_map.add_lines(&mut std::fs::read_to_string("input.txt")?.lines())?;
    oribit_map.validate()?;
    println!(
        "total number of orbits: {}",
        oribit_map.total_number_of_orbits()?
    );
    println!(
        "transfer distance: {}",
        oribit_map.find_distance_between("YOU", "SAN")?
    );
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn do_test() {
        let mut oribit_map = OrbitMap::new();
        oribit_map.add_orbit("A", "B").expect("failed to add entry");
        oribit_map
            .add_orbit("COM", "B")
            .expect_err("failed to fail");
        oribit_map.validate().expect_err("should be invalid");
        oribit_map
            .add_orbit("COM", "A")
            .expect("failed to add entry");
        oribit_map.validate().expect("should be valid");
    }

    #[test]
    fn test_sample() {
        let map_text = "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L";
        let mut map = OrbitMap::new();
        map.add_lines(&mut map_text.lines())
            .expect("failed to add lines");
        assert_eq!(42, map.total_number_of_orbits().expect("failed to count"));
    }

    #[test]
    fn test_sameple_distance() {
        let map_text = "COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L
K)YOU
I)SAN";
        let mut map = OrbitMap::new();
        map.add_lines(&mut map_text.lines())
            .expect("failed to add lines");
        assert_eq!(4, map.find_distance_between("YOU", "SAN").unwrap());
    }
}
