extern crate reqwest;
use std::collections::VecDeque;
use std::fs::{self,File};
use std::io::{Write, BufWriter};
use std::env;
use image::ImageBuffer;
use rand::Rng;
use rand::rngs::ThreadRng;


fn main() {
  fs::create_dir_all("./data").unwrap();
  let args : Vec<String> = env::args().collect();
  let (zoom,lat,long,size) = 
    match parse_args(args) {
      //None => (12,35.75,139.1,2),
      None => panic!("parse error..."),
      Some(t) => t
    };
  //let zoom = 14;
  //let lat  = 35.425847;
  //let long = 137.385063;
  //let size = 4;

  let data = download_data(zoom, lat, long, size);
  let map  = download_map(zoom, lat, long, size);
  //let data = test_data();
  println!("downloaded: {}x{}",data.len(),data[0].len());
  let highest = data.iter().map(|v| v.iter().fold(0.0/0.0, |m, v| v.max(m))).fold(0.0/0.0, |m, v| v.max(m));
  let lowest  = data.iter().map(|v| v.iter().fold(0.0/0.0, |m, v| v.min(m))).fold(0.0/0.0, |m, v| v.min(m));
  println!("min: {}m",lowest);
  println!("max: {}m",highest);
  let gradient = lowest_neighbor_table2(&data);
  let light_table = light(&gradient);
  //let table = lowest_neighbor_table(&data);
  let table = fix_neibor_table(&data,lowest_neighbor_table(&data));
  //let table = remove_sizeN_loop(fix_neibor_table(&data, remove_sizeN_loop(fix_neibor_table(&data, remove_sizeN_loop(gradient)))));
  let (system,count,ends) = move_water(&table);
  let (_,x,y) = get_id(zoom,lat,long);
  //write_csv("./data/id.csv"    ,&vec![vec![zoom.to_string(),x.to_string(),y.to_string(),size.to_string()]])  .unwrap_or(());
  //write_csv("./data/data.csv"  ,&data)  .unwrap_or(());
  //write_csv("./data/system.csv",&system).unwrap_or(());
  //write_csv("./data/count.csv" ,&count) .unwrap_or(());
  //write_csv("./data/light.csv" ,&light_table) .unwrap_or(());
  //write_csv("./data/ends.csv"  ,&ends.iter().map(|(x,y)| vec![x,y]).collect()) .unwrap_or(());

  let mut rng = rand::thread_rng();
  let colors:Vec<[u8;3]> = ends.iter().map(|(_,_)| random_color(&mut rng)).collect();

  let h = data.len();
  let w = data[0].len();

  let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
    image::Rgb(map[y as usize][x as usize])
  }).save("data/map.png");

  let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
    let l = ((data[y as usize][x as usize] - lowest) / (highest - lowest) * 255.0) as u8;
    image::Rgb([l, l, l])
  }).save("data/height.png");

  let max_light = light_table.iter().map(|v| v.iter().fold(0.0/0.0, |m, v| v.max(m))).fold(0.0/0.0, |m, v| v.max(m));
  let min_light = light_table.iter().map(|v| v.iter().fold(0.0/0.0, |m, v| v.min(m))).fold(0.0/0.0, |m, v| v.min(m));
  let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
    let l = f64::floor((f64::min(1.0,light_table[y as usize][x as usize] - min_light) / (max_light - min_light)) * 255.0) as u8;
    image::Rgb([l, l, l])
  }).save("data/light.png");

  let system_colored: Vec<Vec<[u8;3]>> = system.iter().map(|v| v.iter().map( |s|
    if *s == -1 {
      [255,255,255]
    }else{
      colors[*s as usize]
    }).collect()).collect();

  let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
    image::Rgb(system_colored[y as usize][x as usize])
  }).save("data/systems.png");

  let pretty_count : Vec<Vec<f64>> = count.iter().map(|v| v.iter().map(|c| f64::powf(*c as f64,0.2)).collect()).collect();

  let max_pcnt = pretty_count.iter().map(|v| v.iter().fold(0.0/0.0, |m, v| v.max(m))).fold(0.0/0.0, |m, v| v.max(m));
  let min_pcnt = pretty_count.iter().map(|v| v.iter().fold(0.0/0.0, |m, v| v.min(m))).fold(0.0/0.0, |m, v| v.min(m));
  
  let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
    let l = f64::floor((f64::min(1.0,pretty_count[y as usize][x as usize] - min_pcnt) / (max_pcnt/4.0 - min_pcnt)) * 255.0) as u8;
    image::Rgb([l, l, l])
  }).save("data/river.png");

  let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
    let l = f64_clip((pretty_count[y as usize][x as usize] - min_pcnt) / (max_pcnt/4.0 - min_pcnt),0.0,1.0);
    let r = f64_clip(system_colored[y as usize][x as usize][0] as f64,0.0,255.0) as u8;
    let g = f64_clip(system_colored[y as usize][x as usize][1] as f64,0.0,255.0) as u8;
    let b = f64_clip(system_colored[y as usize][x as usize][2] as f64,0.0,255.0) as u8;
    let a = f64_clip(l * 255.0 ,0.0,255.0) as u8;
    image::Rgba([r, g, b, a])
  }).save("data/colored.png");
  
  let _ = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
    let l = f64_clip((pretty_count[y as usize][x as usize] - min_pcnt) / (max_pcnt/4.0 - min_pcnt),0.0,1.0);
    let r = f64_clip(system_colored[y as usize][x as usize][0] as f64 * l + map[y as usize][x as usize][0] as f64 * (1.0-l) ,0.0,255.0) as u8;
    let g = f64_clip(system_colored[y as usize][x as usize][1] as f64 * l + map[y as usize][x as usize][1] as f64 * (1.0-l) ,0.0,255.0) as u8;
    let b = f64_clip(system_colored[y as usize][x as usize][2] as f64 * l + map[y as usize][x as usize][2] as f64 * (1.0-l) ,0.0,255.0) as u8;
    image::Rgb([r, g, b])
  }).save("data/result.png");
  println!("{}",get_url_png_from_ll(zoom,lat,long));
}

fn f64_clip(x:f64,min:f64,max:f64) -> f64 {
  f64::max(min,f64::min(x,max))
}

fn random_color(rng: &mut ThreadRng) -> [u8;3] {
  let r:u8 = rng.gen();
  let g:u8 = rng.gen();
  let b:u8 = rng.gen();
  [r/2+128,g/2+128,b/2+128]
}

fn parse_args(args : Vec<String>) -> Option<(i32,f64,f64,i32)> {
  if args.len()>4 {
    let zoom : Option<i32> = args[1].parse().ok();
    let lat  : Option<f64> = args[2].parse().ok();
    let long : Option<f64> = args[3].parse().ok();
    let size : Option<i32> = args[4].parse().ok();
    Some((zoom?,lat?,long?,size?))
  }else{
    None
  }
}

fn test_data() -> Vec<Vec<f64>>{
  let mut r = vec![vec![0.0;64];64];
  r[32][0] = -1.0;
  r
}

fn write_csv(path:&str, data:&Vec<Vec<impl ToString>>) -> Result<(), Box<dyn std::error::Error>> {
  let mut writer = BufWriter::new(File::create(path)?);
  for r in data {
      let strr: Vec<String> = r.iter().map(|x| x.to_string()).collect();
      let joined = strr.join(",")+"\n";
      writer.write_all(&joined.as_bytes())?;
  }
  writer.flush()?;
  Ok(())
}

fn light(table: &Vec<Vec<(i32,i32)>>) -> Vec<Vec<f64>>{
  println!("making neighbor table...");
  let mut result = Vec::new();
  let h = table.len()    as i32;
  let w = table[0].len() as i32;
  for _y in 0..h {
    print!("{}/{}\r",_y.to_string(),h.to_string());
    let mut r = Vec::new();
    for _x in 0..w {
      let (dy,dx) = table[_y as usize][_x as usize];
      r.push(0.5-((dy-_y)+(dx-_x)) as f64/2.0/2.0);
    }
    result.push(r);
  }
  result
}


const MIN_HEIGHT:f64 = 0.0;

fn lowest_neighbor_table(data: &Vec<Vec<f64>>) -> Vec<Vec<(i32,i32)>>{
  println!("making neighbor table...");
  let mut result = Vec::new();
  let h = data.len()    as i32;
  let w = data[0].len() as i32;
  let size = 20;
  let f = |i:i32,j:i32,mh:&mut f64,mc:&mut (i32,i32)| {
    if between(0, i, h-1) && between(0, j, w-1) && data[i as usize][j as usize] < *mh {
      *mh = data[i as usize][j as usize];
      *mc = (i,j);
    }
  };
  for _y in 0..h {
    print!("{}/{}\r",_y.to_string(),h.to_string());
    let mut r = Vec::new();
    for _x in 0..w {
      let mhinit = data[_y as usize][_x as usize];
      let mut mh = mhinit;
      let mut mc = (_y,_x);
      for s in 1..size+1 {
        let i = _y-s;
        for j in (_x-s)..(_x+(s-1))+1 {
          f(i,j,&mut mh,&mut mc);
        }
        let j = _x+s;
        for i in (_y-s)..(_y+(s-1))+1 {
          f(i,j,&mut mh,&mut mc);
        }
        let i = _y+s;
        for j in (_x-(s-1))..(_x+s)+1 {
          f(i,j,&mut mh,&mut mc);
        }
        let j = _x-s;
        for i in (_y-(s-1))..(_y+s)+1 {
          f(i,j,&mut mh,&mut mc);
        }
        
        if mhinit-mh<MIN_HEIGHT {
          mh = mhinit;
          mc = (_y,_x);
        }
        if mc != (_y,_x) {
          break;
        }
      }
      r.push(mc);
    }
    result.push(r);
  }
  println!("");
  println!("done.");
  result
}

fn sigint(x:f64) -> i32{
  if x>0.0{
    1
  }else if x<0.0{
    -1
  }else{
    0
  }
}

fn lowest_neighbor_table2(data: &Vec<Vec<f64>>) -> Vec<Vec<(i32,i32)>>{
  println!("making neighbor table...");
  let mut result = Vec::new();
  let h = data.len()    as i32;
  let w = data[0].len() as i32;
  let size = 3;
  let mut exptable : Vec<f64> = Vec::new();
  for i in 0..(size*size*2)+1 {
    exptable.push(std::f64::consts::E.powf(-i as f64));
  }
  let mut c =0;
  for _y in 0..h {
    print!("{}/{}\r",_y.to_string(),h.to_string());
    let mut r = Vec::new();
    for _x in 0..w {
      let mut sum_weight = 0.0;
      let mut sumx = 0.0;
      let mut sumy = 0.0;
      for y in i32::max(_y-size,0)..i32::min(_y+size,h-1)+1 {
        for x in i32::max(_x-size,0)..i32::min(_x+size,w-1)+1 {
          let weight = exptable[((y-_y)*(y-_y) + (x-_x)*(x-_x)) as usize];
          sum_weight += weight;
          if y != _y {
            sumy += (data[y as usize][x as usize]-data[_y as usize][_x as usize]) as f64/(y-_y) as f64 *weight;
          }
          if x != _x {
            sumx += (data[y as usize][x as usize]-data[_y as usize][_x as usize]) as f64/(x-_x) as f64 *weight;
          }
        }
      }
      let mag = 2.0;
      let (ry,rx) = 
        if (sumx*sumx+sumy*sumx)/(sum_weight*sum_weight) > mag*mag{
          (_y-(sumy/sum_weight/mag) as i32,_x-(sumx/sum_weight/mag) as i32)
        }else{
          if sumy == 0.0 && sumx == 0.0 {
            (_y,_x)
          }else if f64::abs(sumy)<f64::abs(sumx){
            if f64::abs(sumy)*2.0<f64::abs(sumx){
              (_y,_x-sigint(sumx))
            }else{
              (_y-sigint(sumy),_x-sigint(sumx))
            }
          }else{
            if f64::abs(sumx)*2.0<f64::abs(sumy){
              (_y-sigint(sumy),_x)
            }else{
              (_y-sigint(sumy),_x-sigint(sumx))
            }
          }
        };
      r.push(
        if between(0, ry, h-1) && between(0, rx, w-1){
          if ry != _y || rx != _x{
            c+=1;
          }
          (ry,rx)
        }else{
          (_y,_x)
        }
      );
    }
    result.push(r);
  }
  println!("");
  println!("{}",c);
  println!("done.");
  result
}

fn between(min:i32, x:i32, max:i32) -> bool {
  min <= x && x <= max
}

fn remove_sizeN_loop(_table: Vec<Vec<(i32,i32)>>) -> Vec<Vec<(i32,i32)>>{
  let size = 10;
  println!("removing size-{} loops.",size);
  let mut table = _table;
  let h = table.len()    as i32;
  let w = table[0].len() as i32;

  let mut ecount = 0;
  for _y in 0..h {
    for _x in 0..w {
      if table[_y as usize][_x as usize] == (_y,_x){
        ecount += 1;
      }
    }
  }
  println!("before:{}",ecount);

  let mut flag = vec![vec![false; h as usize]; w as usize];
  let mut maxsize = 0;
  for _y in 0..h {
    print!("{}/{}\r",_y.to_string(),h.to_string());
    for _x in 0..w {
      let mut dest = table[_y as usize][_x as usize] ;
      let mut log = Vec::new();
      log.push(dest);
      for _i in 0..size {
        let (dy,dx) = dest;
        dest = table[dy as usize][dx as usize];
        log.push(dest);
        if dest == (_y,_x) {
          for (_dy,_dx) in log{
            maxsize = i32::max(_i,maxsize);
            flag[_dy as usize][_dx as usize] = true;
          }
          break;
        }
      }
    }
  }
  for _y in 0..h {
    print!("{}/{}\r",_y.to_string(),h.to_string());
    for _x in 0..w {
      if flag[_y as usize][_x as usize]{
        table[_y as usize][_x as usize] = (_y,_x);
      }
    }
  }

  println!("maxsize:{}",maxsize);
  let mut ecount = 0;
  for _y in 0..h {
    for _x in 0..w {
      if table[_y as usize][_x as usize] == (_y,_x){
        ecount += 1;
      }
    }
  }
  println!("after:{}",ecount);
  println!("done.");
  table
}
fn fix_neibor_table(data: &Vec<Vec<f64>>, _table: Vec<Vec<(i32,i32)>>) -> Vec<Vec<(i32,i32)>>{
  println!("searching exits...");
  let mut table = _table;
  let h = table.len()    as i32;
  let w = table[0].len() as i32;
  let mut searched = vec![vec![false; h as usize]; w as usize];
  for _y in 0..h {
    print!("{}/{}\r",_y.to_string(),h.to_string());
    for _x in 0..w {
      if table[_y as usize][_x as usize] == (_y as i32,_x as i32) && !searched[_y as usize][_x as usize] {
        let mut queue = VecDeque::new();
        let mut log   = VecDeque::new();
        let mut exitc = (_y,_x);
        let exithinit = data[_y as usize][_x as usize];
        let mut exith = exithinit;
        queue.push_back((_y,_x));
        log.push_back((_y,_x));
        loop {
          match queue.pop_front() {
            None => break,
            Some((cy,cx)) => {
              if !searched[cy as usize][cx as usize]{
                //print!("{},{},{}\n",cy,cx,searched[cy as usize][cx as usize] );
                searched[cy as usize][cx as usize] = true;
                let (dy,dx) = table[cy as usize][cx as usize];
                if (dy,dx) == (cy,cx) /*|| f64::abs(data[dy as usize][dx as usize]-exithinit)<MIN_HEIGHT*/ {
                  log.push_back((dy,dx));
                  for ny in cy-1..cy+1+1 {
                    for nx in cx-1..cx+1+1 {
                      if between(0,ny as i32,h as i32 -1) && between(0, nx as i32, w as i32 -1) && !searched[ny as usize][nx as usize] {
                        queue.push_back((ny,nx));
                      }
                    }
                  }
                }else{
                }
                if data[dy as usize][dx as usize] < exith {
                  exitc = (dy,dx);
                  exith = data[dy as usize][dx as usize];
                }
              }
            }
          }
        }
        for (cy,cx) in log{
          table[cy as usize][cx as usize] = exitc;
        }
      }
    }
  }
  println!("");
  println!("done.");
  table
}

fn move_water(table: &Vec<Vec<(i32,i32)>>) -> (Vec<Vec<i32>>,Vec<Vec<i32>>,Vec<(i32,i32)>){
  println!("moving water.");
  let h = table.len();
  let w = table[0].len();
  let mut ends : Vec<(i32,i32)> = Vec::new();
  let mut children : Vec<Vec<Vec<(i32,i32)>>> = vec![vec![Vec::new();w];h];
  for y in 0..h {
    print!("{}/{}\r",y.to_string(),h.to_string());
    for x in 0..w {
      let (cy,cx) = table[y][x];
      if cy==y as i32 && cx==x as i32 {
        ends.push((y as i32,x as i32));
      }else{
        children[cy as usize][cx as usize].push((y as i32,x as i32));
      } 
    }
  }
  print!("\n");
  let mut system : Vec<Vec<i32>> = vec![vec![-1;w];h];
  let mut count  : Vec<Vec<i32>> = vec![vec![0;w];h];
  let mut csum = 0;

  for (i,e) in ends.iter().enumerate() {
    let mut queue  = VecDeque::new();
    let mut sorted = VecDeque::new();
    queue .push_back(e);
    sorted.push_back(e);
    loop {
      match queue.pop_front() {
        None => break,
        Some(p) => {
          let (py,px) = p;
          system[*py as usize][*px as usize] = i as i32;
          count [*py as usize][*px as usize] += 1;
          for c in &children[*py as usize][*px as usize] {
            queue .push_back(&c);
            sorted.push_back(&c);
          }
        }
      }
    }
    for (py,px) in sorted.iter().rev() {
      for (cy,cx) in &children[*py as usize][*px as usize] {
        count[*py as usize][*px as usize] += count[*cy as usize][*cx as usize];
      }
    }
    let (ey,ex) = e;
    csum += count[*ey as usize][*ex as usize];
  }
  println!("{}",ends.len());
  println!("{}/{} {}%",csum,w*h,f64::floor((csum as f64/(w*h) as f64)*100.0));
  println!("done!");
  (system,count,ends)
}

fn get_id(zoom:i32, lat:f64, long:f64) -> (i32,i32,i32) {
  let pi = std::f64::consts::PI;
  let l = 85.05112878;
  let x = (f64::powf(2.0,(zoom-1) as f64) * (long/180.0 +1.0) as f64).floor() as i32;
  let y = (f64::powf(2.0,(zoom-1) as f64) / pi*(f64::atanh(f64::sin(l/180.0*pi))-f64::atanh(f64::sin(lat/180.0*pi)))).floor() as i32;
  return (zoom,x,y)
}

fn get_url(zoom:i32,x:i32,y:i32) -> String {
  "https://cyberjapandata.gsi.go.jp/xyz/dem/".to_string()+&(zoom.to_string())+"/"+&(x.to_string())+"/"+&(y.to_string())+".txt"
}

fn get_url_png(zoom:i32,x:i32,y:i32) -> String {
  "https://cyberjapandata.gsi.go.jp/xyz/std/".to_string()+&(zoom.to_string())+"/"+&(x.to_string())+"/"+&(y.to_string())+".png"
}
fn get_url_png_from_ll(zoom:i32, lat:f64, long:f64) -> String {
  let (_,x,y) = get_id(zoom,lat,long);
  get_url_png(zoom,x,y)
}

fn string_to_tile(str:&str) -> Vec<Vec<f64>> {
  let mut result = Vec::new();
  for row in str.split('\n') {
    if row == "" {
      continue;
    }
    let mut rv = Vec::new();
    for col in row.split(',') {
      rv.push(match col.parse() {
        Ok(v)  => v,
        Err(_) => 0.0 
      });
    }
    if rv.len() != 256 {
      println!(" invalid data: {}cols",rv.len());
      return vec![vec![0.0;256];256];
    }
    result.push(rv);
  }
  if result.len() != 256 {
    println!(" invalid data: {}cols",result.len());
    return vec![vec![0.0;256];256];
  }
  result
}

fn download_maptile(zoom:i32, x:i32, y:i32) -> Vec<Vec<[u8; 3]>>{
  let url = get_url_png(zoom,x,y);
  print!("\rdownloading: {}",url);

  let img = image::load_from_memory(&reqwest::blocking::get(url).unwrap().bytes().unwrap()).unwrap().into_rgb8();
  let mut raw = vec![];
  for row in img.rows() {
    let mut rd = vec![];
    for px in row {
      rd.push(px.0);
    }
    raw.push(rd);
  }
  raw
}

fn download_map(zoom:i32, lat:f64, long:f64, size:i32) -> Vec<Vec<[u8;3]>> {
  let mut tiles : Vec<Vec<Vec<Vec<[u8;3]>>>> = Vec::new();
  let (zoom,tx,ty) = get_id(zoom, lat, long);
  for i in -size..size+1 {
    let mut tr = Vec::new();
    for j in -size..size+1 {
      tr.push(download_maptile(zoom,tx+j,ty+i));
    }
    tiles.push(tr);
  }
  println!("");

  let mut result : Vec<Vec<[u8;3]>> = Vec::new();
  for j in 0..size*2+1 {
    for y in 0..256 {
      let mut rr = Vec::new();
      for i in 0..size*2+1 {
        rr.append(&mut tiles[j as usize][i as usize][y as usize])
      }
      result.push(rr);
    }
  }
  result
}

fn zeros256x256() -> Vec<Vec<f64>> {
  vec![vec![0.0; 256]; 256]
}


fn download_tile(zoom:i32, x:i32, y:i32) -> Vec<Vec<f64>> {
  let url = get_url(zoom,x,y);
  print!("\rdownloading: {}",url);
  match reqwest::blocking::get(url) {
    Ok(r)  => 
    match r.text(){
      Ok(t) => string_to_tile(&t),
      Err(_) => zeros256x256()
    } 
    Err(_) => zeros256x256()
  }
}

fn download_data(zoom:i32, lat:f64, long:f64, size:i32) -> Vec<Vec<f64>> {
  let mut tiles : Vec<Vec<Vec<Vec<f64>>>> = Vec::new();
  let (zoom,tx,ty) = get_id(zoom, lat, long);
  for i in -size..size+1 {
    let mut tr = Vec::new();
    for j in -size..size+1 {
      tr.push(download_tile(zoom,tx+j,ty+i));
    }
    tiles.push(tr);
  }
  println!("");

  let mut result : Vec<Vec<f64>> = Vec::new();
  for j in 0..size*2+1 {
    for y in 0..256 {
      let mut rr = Vec::new();
      for i in 0..size*2+1 {
        rr.append(&mut tiles[j as usize][i as usize][y as usize])
      }
      result.push(rr);
    }
  }
  result
}

//fn gauss_filter(data:&Vec<Vec<f64>>, size:i32) -> Vec<Vec<f64>> {
//  let mut copied : Vec<Vec<f64>> = data.clone();
//  for i in -size..size+1 {
//    let mut tr = Vec::new();
//    for j in -size..size+1 {
//      tr.push(download_tile(zoom,tx+j,ty+i));
//    }
//    println!("");
//    tiles.push(tr);
//  }
//
//  let mut result : Vec<Vec<f64>> = Vec::new();
//  for j in 0..size*2+1 {
//    for y in 0..256 {
//      let mut rr = Vec::new();
//      for i in 0..size*2+1 {
//        rr.append(&mut tiles[j as usize][i as usize][y as usize])
//      }
//      result.push(rr);
//    }
//  }
//  result
//}