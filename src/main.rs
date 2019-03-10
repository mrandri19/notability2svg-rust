extern crate base64;
extern crate byteorder;
extern crate roxmltree;
extern crate svg;

use svg::node::element::Path;
use svg::Document;

use std::env;
use std::fs;
use std::io::{Cursor, Read};

use byteorder::{NativeEndian, ReadBytesExt};

use base64::decode;

fn load_file(path: &str) -> String {
    let mut file = fs::File::open(&path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}

fn parse_session(file: &str) -> (Vec<f32>, Vec<i32>, Vec<f32>, Vec<u32>) {
    let mut curvesnumpoints_base64 = None;
    let mut curvespoints_base64 = None;
    let mut curveswidth_base64 = None;
    let mut curvescolors_base64 = None;

    let doc = roxmltree::Document::parse(&file).unwrap();
    for node in doc.descendants() {
        if node.node_type() == roxmltree::NodeType::Text {
            if node.text().unwrap() == "curvespoints" {
                curvespoints_base64 = Some(
                    node.parent()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .first_child()
                        .unwrap()
                        .text()
                        .unwrap()
                        .trim()
                        .replace("\n", "")
                        .replace("\t", ""),
                )
            }
            if node.text().unwrap() == "curvesnumpoints" {
                curvesnumpoints_base64 = Some(
                    node.parent()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .first_child()
                        .unwrap()
                        .text()
                        .unwrap()
                        .trim()
                        .replace("\n", "")
                        .replace("\t", ""),
                )
            }
            if node.text().unwrap() == "curveswidth" {
                curveswidth_base64 = Some(
                    node.parent()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .first_child()
                        .unwrap()
                        .text()
                        .unwrap()
                        .trim()
                        .replace("\n", "")
                        .replace("\t", ""),
                )
            }
            if node.text().unwrap() == "curvescolors" {
                curvescolors_base64 = Some(
                    node.parent()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .first_child()
                        .unwrap()
                        .text()
                        .unwrap()
                        .trim()
                        .replace("\n", "")
                        .replace("\t", ""),
                )
            }
        }
    }

    let mut curvesnumpoints = vec![];
    let curvesnumpoints_bytes = decode(&curvesnumpoints_base64.unwrap()).unwrap();
    let mut rdr = Cursor::new(curvesnumpoints_bytes);
    while let Ok(v) = rdr.read_i32::<NativeEndian>() {
        curvesnumpoints.push(v);
    }

    let mut curvespoints = vec![];
    let curvespoints_bytes = decode(&curvespoints_base64.unwrap()).unwrap();
    let mut rdr = Cursor::new(curvespoints_bytes);
    while let Ok(v) = rdr.read_f32::<NativeEndian>() {
        curvespoints.push(v);
    }
    assert!(curvespoints.len() % 2 == 0);

    let mut curveswidth = vec![];
    let curveswidth_bytes = decode(&curveswidth_base64.unwrap()).unwrap();
    let mut rdr = Cursor::new(curveswidth_bytes);
    while let Ok(v) = rdr.read_f32::<NativeEndian>() {
        curveswidth.push(v);
    }

    let mut curvescolors = vec![];
    let curvescolors_bytes = decode(&curvescolors_base64.unwrap()).unwrap();
    let mut rdr = Cursor::new(curvescolors_bytes);
    while let Ok(v) = rdr.read_u32::<NativeEndian>() {
        curvescolors.push(v);
    }

    (curvespoints, curvesnumpoints, curveswidth, curvescolors)
}

fn transform_u32_to_array_of_u8_le(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;

    // Little endian so LSB last
    return [b4, b3, b2, b1];
}

fn find_max_point_y(v: &[f32]) -> f32 {
    *v.iter()
        .skip(1)
        .step_by(2)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

fn draw(
    points: Vec<f32>,
    points_per_curve: Vec<i32>,
    widths: Vec<f32>,
    colors: Vec<u32>,
) -> svg::Document {
    let width = 565;
    let height = find_max_point_y(&points) + width as f32 * (16. / 9.) as f32;

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", width)
        .set("height", height);

    let mut points_so_far: usize = 0;

    let mut curve_index = 0;
    for num_points in points_per_curve {
        let [r, g, b, a] = transform_u32_to_array_of_u8_le(colors[curve_index]);
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", format!("rgb({},{},{})", r, g, b))
            .set("stroke-linecap", "round")
            .set("stroke-linejoin", "round")
            .set("stroke-opacity", (a as f32) / 255.)
            .set("stroke-width", widths[curve_index]);
        let mut d = String::new();
        {
            let x = points[points_so_far * 2 + 0];
            let y = points[points_so_far * 2 + 1];
            d.push_str(&format!("M{} {} ", x, y));
        }
        for i in (points_so_far + 1)..(points_so_far + num_points as usize) {
            let x = points[i * 2];
            let y = points[i * 2 + 1];
            d.push_str(&format!("L{} {} ", x, y));
        }

        document = document.add(path.set("d", d));

        points_so_far += num_points as usize;
        curve_index += 1;
    }

    document
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} input.plist.xml output.svg", args[0]);
        std::process::exit(1);
    }

    let session = load_file(&args[1]);
    let (points, points_per_curve, widths, colors) = parse_session(&session);
    let document = draw(points, points_per_curve, widths, colors);

    svg::save(&args[2], &document).unwrap();
}
