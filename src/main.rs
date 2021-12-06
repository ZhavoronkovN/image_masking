use std::convert::TryInto;
use image::io::Reader as ImageReader;
use image::Rgb;
use std::collections::HashMap;

fn main() {
    let _select_red = |pix : &Rgb<u8>| pix[0];
    let _select_green = |pix : &Rgb<u8>| pix[1];
    let _select_blue = |pix : &Rgb<u8>| pix[2];
    let _select_overall_brightness = |pix : &Rgb<u8>| calc_brightness(pix);
    println!("Default hist : ");
    draw_hist(&"images/defaults/london.jpg".to_string(), &_select_blue, 300, 10);
    normalize_hist(&"images/defaults/london.jpg".to_string(),&"images/hist_eq/london_red.jpg".to_string(),&_select_red);
    normalize_hist(&"images/defaults/london.jpg".to_string(),&"images/hist_eq/london_blue.jpg".to_string(),&_select_blue);
    normalize_hist(&"images/defaults/london.jpg".to_string(),&"images/hist_eq/london_green.jpg".to_string(),&_select_green);
    println!("Hist after equalization : ");
    draw_hist(&"images/hist_eq/london_blue.jpg".to_string(), &_select_blue, 300, 10);
    println!("\n===============================\n");
    let _roberts = vec![vec![-1.0,0.0,0.0,1.0],vec![0.0,-1.0,1.0,0.0]];
    let _previt = vec![vec![1.0,0.0,-1.0,1.0,0.0,-1.0,1.0,0.0,-1.0],vec![-1.0,-1.0,-1.0,0.0,0.0,0.0,1.0,1.0,1.0]];
    let _sobel = vec![vec![1.0,0.0,-1.0,2.0,0.0,-2.0,1.0,0.0,-1.0],vec![-1.0,-2.0,-1.0,0.0,0.0,0.0,1.0,2.0,1.0]];
    println!("Applying Roberts operator...\n");
    apply_mask(&"images/defaults/default.bmp".to_string(),&"images/filters/roberts.bmp".to_string(),&_roberts[1],&_roberts[0],20).expect("Failed to apply roberts mask");
    // println!("Applying Previt operator...\n");
    // apply_mask(&"images/defaults/default.bmp".to_string(),&"images/filters/previt.bmp".to_string(),&_previt[0],&_previt[1],8).expect("Failed to apply previt mask");
    // println!("Applying Sobel operator...\n");
    // apply_mask(&"images/defaults/default.bmp".to_string(),&"images/filters/sobel.bmp".to_string(),&_sobel[0],&_sobel[1],4).expect("Failed to apply sobel mask");
    println!("Finished");
}

fn to_pix(d : &[f32;3]) -> Rgb<u8>
{
    let mut res = Rgb([0,0,0]);
    for i in 0..3
    {
        res[i] = d[i].abs() as u8;
    }
    res
}

fn to_pix_bw(d : &[f32;3], m : u16) -> Rgb<u8>
{
    let mut sum = 0.0;
    for i in 0..3
    {
        sum += d[i].abs();
    }
    let val = match ((sum/3.0) as u16 * m) > 255{true => 255 as u8, false => ((sum/3.0) as u16 * m) as u8 };
    Rgb([val; 3])
}

fn add(a : &[f32;3], b : &[f32;3]) -> [f32;3]
{
    let mut data : [f32;3] = [0.0;3];
    for i in 0..3
    {
        data[i] = a[i] + b[i];
    }
    data
}

fn multiply(a : &Rgb<u8>, b : f32) -> [f32;3]
{
    let mut data : [f32;3] = [0.0;3];
    for i in 0..3
    {
        data[i] = a[i] as f32 * b;
    }
    data
}

fn sqrtpix2(a : &Rgb<u8>, b : &Rgb<u8>) -> Rgb<u8>
{
    let mut res = Rgb([0,0,0]);
    for i in 0..3
    {
        res[i] = (((a[i] as i32).pow(2) + (b[i] as i32).pow(2)) as f32).sqrt() as u8;
    }
    res
}

fn apply_mask_for_pixel(image : &image::ImageBuffer<Rgb<u8>,Vec<u8>>, mask : &Vec<f32>, x : u32, y : u32, multiply_on : u16) -> Result<Rgb<u8>, String>
{
    if mask.iter().sum::<f32>() as i8 != 0
    {
        return Err("Sum of mask should be null".to_string());
    }
    let mask_size = (mask.len() as f32).sqrt() as usize;
    let mut new_pix : [f32;3] = [0.0;3];
    for i in 0..mask_size
    {
        for j in 0..mask_size
        {
            new_pix = add(&new_pix, &multiply(image.get_pixel(x + j as u32, y + i as u32),mask[i * mask_size + j]));
        }
    }
    Ok(to_pix_bw(&new_pix,multiply_on))
}

fn apply_mask(image : &String, outimage : &String, maskx : &Vec<f32>, masky : &Vec<f32>, multiply_on : u16) -> Result<(),String>
{
    let mut img = ImageReader::open(image.as_str()).unwrap().decode().unwrap().to_rgb8();
    for y in 0..img.height()-maskx.len() as u32 + 1
    {
        for x in 0..img.width()-maskx.len() as u32 + 1
        {
            img.put_pixel(x,y,sqrtpix2(&apply_mask_for_pixel(&img, maskx, x, y, multiply_on)?,&apply_mask_for_pixel(&img, masky, x, y, multiply_on)?));
        }
    }
    img.save(outimage).unwrap();
    Ok(())
}

fn calc_brightness(pix : &Rgb<u8>) -> u8
{
    (0.3 * pix[0] as f32 + 0.59 * pix[1] as f32 + 0.11 * pix[2] as f32) as u8
}

fn normalize_hist(image : &String, outimage : &String, mode : &dyn Fn(&Rgb<u8>) -> u8)
{
    let img = ImageReader::open(image.as_str()).unwrap().decode().unwrap().to_rgb8();
    let pixel_count = img.width()*img.height();
    let mut output = img.clone();
    let mut pi  = HashMap::new();
    
    let mut sorted : Vec<&Rgb<u8>> = output.pixels().collect();
    sorted.sort_by(|a, b|
        {
            mode(&a).partial_cmp(&mode(&b)).unwrap()
        });
    let min = mode(sorted.first().unwrap());
    let max = mode(sorted.last().unwrap());
    let mut last_pix = sorted[1].clone();
    let mut count = 1;
    for pix in sorted
    {
        if mode(pix) == mode(&last_pix)
        {
            count += 1;
        }
        else
        {
            pi.insert(mode(pix),count as f32 / pixel_count as f32);
            last_pix = pix.clone();
        }
    }

    for pix in output.pixels_mut()
    {
        if !pi.contains_key(&mode(&pix))
        {
            continue;
        }
        let k = *pi.get(&mode(&pix)).expect(format!("No such value in map : {}. Pix : {:?}",&mode(&pix),&pix).as_str());

        for i in 0..3
        {
            let new_val = pix[i] as f32 * (k * (max - min) as f32 + min as f32) / mode(&pix) as f32;
            pix[i] = match new_val > 255.0 {true => 255.0, false => new_val} as u8;
        }
    }
    output.save(outimage).unwrap();
}


fn draw_hist(image : &String, mode : &dyn Fn(&Rgb<u8>) -> u8, max_width : u16, height : u8)
{
    let img = ImageReader::open(image.as_str()).unwrap().decode().unwrap().to_rgb8();
    let len = img.width()*img.height();
    let mut output = img.clone();
    
    let mut sorted : Vec<&Rgb<u8>> = output.pixels().collect();
    sorted.sort_by(|a, b|
        {
            mode(&a).partial_cmp(&mode(&b)).unwrap()
        });
    let mut uniq_data = sorted.clone();
    uniq_data.dedup_by(|a,b| mode(&a) == mode(&b));
    let values_per_row = (uniq_data.len() as f64 / height as f64) as i64 - 1;
    let mut count = 1 as usize;
    let mut count_sum = 0 as usize;
    let mut height_count = 0 as usize;
    let mut last_pix = sorted[1].clone();
    for pix in sorted
    {
        if mode(pix) == mode(&last_pix)
        {
            count += 1;
        }
        else
        {
            height_count += 1;
            count_sum += count;
            if height_count == values_per_row.try_into().unwrap()
            {   
                println!("{:#<1$}","",((count_sum as f64 / len as f64 * max_width as f64) + 1.0) as usize);
                height_count = 0;
                count_sum = 0;
            }
            count = 1;
            last_pix = pix.clone();
        }
    }
}