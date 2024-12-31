fn main() {
    let str = include_str!("input.txt");
    let nums: Vec<_> = str.chars().map(|ch| ch.to_string().parse::<i8>()).collect();
    println!("Hello, world!");
}
