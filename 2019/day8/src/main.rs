use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum ErrorCodes {
    WrongDimension,
    FileParseFailure,
}

impl fmt::Display for ErrorCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCodes::WrongDimension => write!(f, "wrong dimension"),
            ErrorCodes::FileParseFailure => write!(f, "file parse failure"),
        }
    }
}

impl Error for ErrorCodes {}

fn parse_image_layers(
    width: usize,
    height: usize,
    input: &str,
) -> Result<Vec<Vec<u8>>, ErrorCodes> {
    let pixles_per_layer = width * height;

    let mut all_bytes: Vec<u8> = vec![];
    for ch in input.chars() {
        if let Ok(num) = u8::from_str_radix(&ch.to_string(), 10) {
            all_bytes.push(num);
        } else {
            return Err(ErrorCodes::FileParseFailure);
        }
    }

    Ok(all_bytes
        .chunks_exact(pixles_per_layer)
        .map(|chunck| chunck.iter().map(|c| *c).collect::<Vec<u8>>())
        .collect())
}

struct ColorOccurences {
    pub colors: [usize; 10],
}

fn count_colors(layer: &Vec<u8>) -> ColorOccurences {
    let mut colors: [usize; 10] = [0; 10];

    for color in layer {
        colors[*color as usize] += 1;
    }

    ColorOccurences { colors: colors }
}

fn main() -> Result<(), Box<dyn Error>> {
    let width = 25;
    let height = 6;
    let input = std::fs::read_to_string("input.txt")?;

    let layers = parse_image_layers(width, height, &input)?;

    let mut least_zeros = usize::MAX;
    let mut one_times_two = 0;

    for layer in layers.iter().map(|l| count_colors(&l)) {
        if layer.colors[0] < least_zeros {
            least_zeros = layer.colors[0];
            one_times_two = layer.colors[1] * layer.colors[2];
        }
    }

    println!("{}", one_times_two);

    let decoded_images: Vec<u8> = vec![255; width * height]; //arbitrary high color
    let decoded_images = layers.iter().rev().fold(decoded_images, |img, layer| {
        return img
            .iter()
            .zip(layer.iter())
            .map(|(c_img, c_layer)| match *c_layer {
                0 => 0,
                1 => 1,
                2 => *c_img,
                _ => panic!(),
            })
            .collect();
    });

    for line in decoded_images.chunks_exact(width) {
        for ch in line.iter() {
            print!(
                "{}",
                match ch {
                    0 => ' ',
                    1 => '#',
                    _ => panic!(),
                }
            );
        }
        println!();
    }

    Ok(())
}
