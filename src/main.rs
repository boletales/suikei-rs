#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]

#[macro_use]
extern crate rocket;

extern crate base64;
extern crate reqwest;
use image::ImageBuffer;
use pnet::datalink;
use rocket::{
    config::Config,
    fs::NamedFile,
    response::status::NotFound,
    serde::{json::Json, Serialize},
};
use std::env;
use std::{net::IpAddr, path::Path};
use suikei_rs::*;

#[derive(Serialize)]
struct Images {
    system: String,
    river: String,
    colored: String,
    from: (i32, i32, i32),
    to: (i32, i32, i32),
}

#[get("/")]
async fn index() -> Result<NamedFile, NotFound<String>> {
    let path = Path::new("docs/").join("index.html");
    NamedFile::open(&path)
        .await
        .map_err(|e| NotFound(e.to_string()))
}

#[get("/api/images/<zoom>/<lat>/<long>/<pow>/<mag>")]
fn api_images_pow(zoom: i32, lat: f64, long: f64, pow: f64, mag: f64) -> Json<Images> {
    let zoom = if zoom > 14 { 14 } else { zoom };
    Json(get_images(zoom, lat, long, 1, pow, mag))
}

#[get("/api/images/<zoom>/<lat>/<long>")]
fn api_images(zoom: i32, lat: f64, long: f64) -> Json<Images> {
    let zoom = if zoom > 14 { 14 } else { zoom };
    Json(get_images(zoom, lat, long, 1, 0.2, 4.0))
}

#[launch]
fn rocket() -> _ {
    let mut config = Config::release_default();
    if get_ip_list().len() > 0 {
        config.address = get_ip_list()[0];
    }
    config.port = match env::var("PORT").map(|x| x.parse()) {
        Ok(Ok(p)) => p,
        _ => config.port,
    };
    config.keep_alive = 0;
    rocket::custom(config).mount("/", routes![index, api_images_pow, api_images])
}

fn get_ip_list() -> Vec<IpAddr> {
    let mut ips: Vec<IpAddr> = Vec::new();
    for interface in datalink::interfaces() {
        if !interface.ips.is_empty() && interface.is_up() {
            for ip_net in interface.ips {
                if ip_net.is_ipv4() && !ip_net.ip().is_loopback() {
                    ips.push(ip_net.ip());
                }
            }
        }
    }
    ips
}

fn get_images(zoom: i32, lat: f64, long: f64, size: i32, pow: f64, mag: f64) -> Images {
    let data = download_data(zoom, lat, long, size);
    let table = fix_neibor_table(&data, lowest_neighbor_table(&data));
    let (system, count, ends) = move_water(&table);
    let mut rng = rand::thread_rng();
    let colors: Vec<[u8; 3]> = ends.iter().map(|(_, _)| random_color(&mut rng)).collect();
    let h = data.len();
    let w = data[0].len();

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

    let pretty_count: Vec<Vec<f64>> = count
        .iter()
        .map(|v| v.iter().map(|c| f64::powf(*c as f64, pow)).collect())
        .collect();

    let max_pcnt = pretty_count
        .iter()
        .map(|v| v.iter().fold(f64::NAN, |m, v| v.max(m)))
        .fold(f64::NAN, |m, v| v.max(m));
    let min_pcnt = pretty_count
        .iter()
        .map(|v| v.iter().fold(f64::NAN, |m, v| v.min(m)))
        .fold(f64::NAN, |m, v| v.min(m));

    let system = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        image::Rgb(system_colored[y as usize][x as usize])
    });

    let river = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let l = f64::floor(
            (f64::min(1.0, pretty_count[y as usize][x as usize] - min_pcnt)
                / (max_pcnt / mag - min_pcnt))
                * 255.0,
        ) as u8;
        image::Rgb([l, l, l])
    });

    let colored = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let l = f64_clip(
            (pretty_count[y as usize][x as usize] - min_pcnt) / (max_pcnt / mag - min_pcnt),
            0.0,
            1.0,
        );
        let r = f64_clip(system_colored[y as usize][x as usize][0] as f64, 0.0, 255.0) as u8;
        let g = f64_clip(system_colored[y as usize][x as usize][1] as f64, 0.0, 255.0) as u8;
        let b = f64_clip(system_colored[y as usize][x as usize][2] as f64, 0.0, 255.0) as u8;
        let a = f64_clip(l * 255.0, 0.0, 255.0) as u8;
        image::Rgba([r, g, b, a])
    });

    let rgb_to_base64 = |img: ImageBuffer<image::Rgb<u8>, Vec<u8>>| {
        let mut buf: Vec<u8> = vec![];
        image::DynamicImage::ImageRgb8(img).write_to(&mut buf, image::ImageOutputFormat::Png);
        format!("data:image/png;base64,{}", base64::encode(buf))
    };

    let rgba_to_base64 = |img: ImageBuffer<image::Rgba<u8>, Vec<u8>>| {
        let mut buf: Vec<u8> = vec![];
        image::DynamicImage::ImageRgba8(img).write_to(&mut buf, image::ImageOutputFormat::Png);
        format!("data:image/png;base64,{}", base64::encode(buf))
    };

    let (z, x, y) = get_id(zoom, lat, long);

    Images {
        system: rgb_to_base64(system),
        river: rgb_to_base64(river),
        colored: rgba_to_base64(colored),
        from: (z, x - size, y - size),
        to: (z, x + size, y + size),
    }
}
