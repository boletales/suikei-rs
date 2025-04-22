extern crate reqwest;
use image::ImageBuffer;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::VecDeque;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use suikei_rs::*;

fn main() {
    fs::create_dir_all("./data").unwrap();
    let args: Vec<String> = env::args().collect();
    let (zoom, lat, long, size) = match parse_args(args) {
        //None => (12,35.75,139.1,2),
        None => panic!("parse error..."),
        Some(t) => t,
    };
    //let zoom = 14;
    //let lat  = 35.425847;
    //let long = 137.385063;
    //let size = 4;

    let data = download_data(zoom, lat, long, size);
    let map = download_map(zoom, lat, long, size);
    //let data = test_data();
    println!("downloaded: {}x{}", data.len(), data[0].len());
    let highest = data
        .iter()
        .map(|v| v.iter().fold(f64::NAN, |m, v| v.max(m)))
        .fold(f64::NAN, |m, v| v.max(m));
    let lowest = data
        .iter()
        .map(|v| v.iter().fold(f64::NAN, |m, v| v.min(m)))
        .fold(f64::NAN, |m, v| v.min(m));
    println!("min: {}m", lowest);
    println!("max: {}m", highest);
    let gradient = lowest_neighbor_table2(&data);
    let light_table = light(&gradient);
    //let table = lowest_neighbor_table(&data);
    let table = fix_neibor_table(&data, lowest_neighbor_table(&data));
    //let table = remove_sizeN_loop(fix_neibor_table(&data, remove_sizeN_loop(fix_neibor_table(&data, remove_sizeN_loop(gradient)))));
    let (system, count, ends) = move_water(&table);
    let (_, x, y) = get_id(zoom, lat, long);
    //write_csv("./data/id.csv"    ,&vec![vec![zoom.to_string(),x.to_string(),y.to_string(),size.to_string()]])  .unwrap_or(());
    //write_csv("./data/data.csv"  ,&data)  .unwrap_or(());
    //write_csv("./data/system.csv",&system).unwrap_or(());
    //write_csv("./data/count.csv" ,&count) .unwrap_or(());
    //write_csv("./data/light.csv" ,&light_table) .unwrap_or(());
    //write_csv("./data/ends.csv"  ,&ends.iter().map(|(x,y)| vec![x,y]).collect()) .unwrap_or(());

    let mut rng = rand::thread_rng();
    let colors: Vec<[u8; 3]> = ends.iter().map(|(_, _)| random_color(&mut rng)).collect();

    let h = data.len();
    let w = data[0].len();

    let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        image::Rgb(map[y as usize][x as usize])
    })
    .save("data/map.png");

    let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let l = ((data[y as usize][x as usize] - lowest) / (highest - lowest) * 255.0) as u8;
        image::Rgb([l, l, l])
    })
    .save("data/height.png");

    let max_light = light_table
        .iter()
        .map(|v| v.iter().fold(f64::NAN, |m, v| v.max(m)))
        .fold(f64::NAN, |m, v| v.max(m));
    let min_light = light_table
        .iter()
        .map(|v| v.iter().fold(f64::NAN, |m, v| v.min(m)))
        .fold(f64::NAN, |m, v| v.min(m));
    let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let l = f64::floor(
            (f64::min(1.0, light_table[y as usize][x as usize] - min_light)
                / (max_light - min_light))
                * 255.0,
        ) as u8;
        image::Rgb([l, l, l])
    })
    .save("data/light.png");

    let system_colored: Vec<Vec<[u8; 3]>> = system
        .iter()
        .map(|v| {
            v.iter()
                .map(|s| {
                    if *s == -1 {
                        [255, 255, 255]
                    } else {
                        colors[*s as usize]
                    }
                })
                .collect()
        })
        .collect();

    let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        image::Rgb(system_colored[y as usize][x as usize])
    })
    .save("data/systems.png");

    let pretty_count: Vec<Vec<f64>> = count
        .iter()
        .map(|v| v.iter().map(|c| f64::powf(*c as f64, 0.2)).collect())
        .collect();

    let max_pcnt = pretty_count
        .iter()
        .map(|v| v.iter().fold(f64::NAN, |m, v| v.max(m)))
        .fold(f64::NAN, |m, v| v.max(m));
    let min_pcnt = pretty_count
        .iter()
        .map(|v| v.iter().fold(f64::NAN, |m, v| v.min(m)))
        .fold(f64::NAN, |m, v| v.min(m));

    let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let l = f64::floor(
            (f64::min(1.0, pretty_count[y as usize][x as usize] - min_pcnt)
                / (max_pcnt / 4.0 - min_pcnt))
                * 255.0,
        ) as u8;
        image::Rgb([l, l, l])
    })
    .save("data/river.png");

    let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let l = f64_clip(
            (pretty_count[y as usize][x as usize] - min_pcnt) / (max_pcnt / 4.0 - min_pcnt),
            0.0,
            1.0,
        );
        let r = f64_clip(system_colored[y as usize][x as usize][0] as f64, 0.0, 255.0) as u8;
        let g = f64_clip(system_colored[y as usize][x as usize][1] as f64, 0.0, 255.0) as u8;
        let b = f64_clip(system_colored[y as usize][x as usize][2] as f64, 0.0, 255.0) as u8;
        let a = f64_clip(l * 255.0, 0.0, 255.0) as u8;
        image::Rgba([r, g, b, a])
    })
    .save("data/colored.png");

    let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let l = f64_clip(
            (pretty_count[y as usize][x as usize] - min_pcnt) / (max_pcnt / 4.0 - min_pcnt),
            0.0,
            1.0,
        );
        let r = f64_clip(
            system_colored[y as usize][x as usize][0] as f64 * l
                + map[y as usize][x as usize][0] as f64 * (1.0 - l),
            0.0,
            255.0,
        ) as u8;
        let g = f64_clip(
            system_colored[y as usize][x as usize][1] as f64 * l
                + map[y as usize][x as usize][1] as f64 * (1.0 - l),
            0.0,
            255.0,
        ) as u8;
        let b = f64_clip(
            system_colored[y as usize][x as usize][2] as f64 * l
                + map[y as usize][x as usize][2] as f64 * (1.0 - l),
            0.0,
            255.0,
        ) as u8;
        image::Rgb([r, g, b])
    })
    .save("data/result.png");
    println!("{}", get_url_png_from_ll(zoom, lat, long));
}

fn parse_args(args: Vec<String>) -> Option<(i32, f64, f64, i32)> {
    if args.len() > 4 {
        let zoom: Option<i32> = args[1].parse().ok();
        let lat: Option<f64> = args[2].parse().ok();
        let long: Option<f64> = args[3].parse().ok();
        let size: Option<i32> = args[4].parse().ok();
        Some((zoom?, lat?, long?, size?))
    } else {
        None
    }
}
