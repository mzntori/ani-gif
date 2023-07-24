mod error;

use error::ConvError;

use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::io;

use clap::{Arg, Command};

use gif::{DecodeOptions, ColorOutput};

use gif_dispose;

use riff_ani::{Ani, AniHeader};
use riff_ani::ico::{IconDir, IconDirEntry, IconImage, ResourceType};

#[derive(PartialOrd, PartialEq, Debug)]
enum DebugMode {
    Default,
    Debug,
}

fn gif_to_ani(g: &String, a: &String, frame_rate: u32, hotspot: &String, debug_mode: DebugMode) -> Result<(), ConvError> {
    let gif_path: &Path = Path::new(g);
    let ani_path: &Path = Path::new(a);
    
    // Path logic
    if !gif_path.is_file() {
        return Err(ConvError::InvalidGifPath(g.to_string()));
    }
    
    if ani_path.is_file() {
        println!(
            "{} already exists. Do you want to overwrite it. [y/N] ",
            ani_path
                .file_name()
                .unwrap_or(
                    gif_path
                        .with_extension("ani")
                        .file_name()
                        .unwrap()
                )
                .to_str()
                .unwrap_or("*Invalid Path*")
        );
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        
        if !(user_input.trim().to_lowercase() == "y") {
            return Err(ConvError::UserInputError);
        }
        
        println!("Starting conversion.")
    }
    
    if gif_path.extension() != Some(OsStr::new("gif")) {
        return Err(ConvError::InvalidGifPath(g.to_string()));
    }
    
    if ani_path.extension() != Some(OsStr::new("ani")) {
        return Err(ConvError::InvalidAniExtension(a.to_string()));
    }
    
    // Decode .gif
    let gif_file = File::open(gif_path).unwrap();
    
    let mut gif_decoder = DecodeOptions::new();
    gif_decoder.set_color_output(ColorOutput::Indexed);
    let mut gif_decoder = gif_decoder.read_info(gif_file)?;
    
    // Setup hotspot
    let (hotspot_x, hotspot_y) = parse_hotspot(hotspot)?;
    
    if hotspot_x > gif_decoder.width() || hotspot_y > gif_decoder.height() {
        return Err(ConvError::HotspotOutOfGifRange(hotspot_x, hotspot_y, gif_decoder.width(), gif_decoder.width()));
    }
    
    if hotspot_x > 256 || hotspot_y > 256 {
        return Err(ConvError::HotspotOutOfMaxRange(hotspot_x, hotspot_y));
    }
    
    // Prepare .ani
    let ani_file_result = File::create(ani_path);
    if ani_file_result.is_err() { return Err(ConvError::FailedFileCreation); }
    let ani_file = ani_file_result.unwrap();
    
    let mut ani_frames: Vec<IconDir> = vec![];
    let mut frame_count: u32 = 0;
    
    let mut screen = gif_dispose::Screen::new_decoder(&gif_decoder);
    
    // Convert frame by frame
    while let Some(gif_frame) = gif_decoder.read_next_frame().unwrap() {
        frame_count += 1;
        screen.blit_frame(&gif_frame)?;
        
        let mut rgb_data: Vec<u8> = vec![];
        
        for pixel in screen.pixels.buf() {
            rgb_data.push(pixel.r);
            rgb_data.push(pixel.g);
            rgb_data.push(pixel.b);
            rgb_data.push(pixel.a);
            
            if debug_mode == DebugMode::Debug {
                println!("{} {} {} {}", pixel.r, pixel.g, pixel.b, pixel.a);
            }
        }
        
        let mut ani_frame = IconImage::from_rgba_data(
            screen.pixels.width() as u32,
            screen.pixels.height() as u32,
            rgb_data,
        );
        ani_frame.set_cursor_hotspot(Some((hotspot_x, hotspot_y)));
        
        let entry_result = IconDirEntry::encode_as_png(&ani_frame);
        let entry: IconDirEntry;
        if entry_result.is_err() {
            return Err(ConvError::FailedPngEncoding);
        } else {
            entry = entry_result.unwrap()
        }
        
        let mut icon_dir = IconDir::new(ResourceType::Cursor);
        icon_dir.add_entry(entry);
        ani_frames.push(icon_dir);
        
        if debug_mode == DebugMode::Default {
            print!("\r{} frames converted...", frame_count);
        }
    }
    if debug_mode == DebugMode::Default {
        print!("\r{} frames converted successfully.", frame_count);
        println!();
    }
    
    let ani = Ani {
        header: AniHeader {
            num_frames: frame_count,
            num_steps: frame_count,
            width: gif_decoder.width() as u32,
            height: gif_decoder.height() as u32,
            frame_rate,
        },
        frames: ani_frames,
    };
    let encoding_result = ani.encode(&ani_file);
    
    if encoding_result.is_err() {
        return Err(ConvError::FailedAniEncoding);
    }
    
    Ok(())
}

fn parse_hotspot(h: &String) -> Result<(u16, u16), ConvError> {
    let mut x = 0;
    let mut y = 0;
    
    let parts = h.split(":");
    
    for (part, i) in parts.zip(0..2) {
        if i == 0 {
            let x_parsed = part.parse();
            if x_parsed.is_err() { return Err(ConvError::InvalidHotspotDefinition); }
            x = x_parsed.unwrap();
        } else {
            let y_parsed = part.parse();
            if y_parsed.is_err() { return Err(ConvError::InvalidHotspotDefinition); }
            y = y_parsed.unwrap();
        }
    }
    
    Ok((x, y))
}

fn main() -> Result<(), ConvError> {
    let matches = Command::new("ani-gif")
        .about("convert .gif to .ani")
        .version("0.1")
        .author("mzntori")
        .subcommand(
            Command::new("convert")
                .short_flag('c')
                .long_flag("convert")
                .about("converts a .gif")
                .arg(
                    Arg::new("gif-path")
                        .short('g')
                        .long("gif-path")
                        .help("path to the gif file to convert")
                        .required(true)
                        .num_args(1)
                )
                .arg(
                    Arg::new("ani-path")
                        .short('a')
                        .long("ani-path")
                        .help("path of the file that gets created")
                        .required(true)
                        .num_args(1)
                )
                .arg(
                    Arg::new("framerate")
                        .short('f')
                        .long("framerate")
                        .help("defines how many 1/60s a frame stays on screen (3 -> 3/60s per frame)")
                        .required(true)
                        .num_args(1)
                )
                .arg(
                    Arg::new("hotspot")
                        .long("hotspot")
                        .help("sets the hotspot of the cursor")
                        .required(false)
                        .num_args(1)
                        .default_value(OsStr::new("0:0"))
                )
        )
        .get_matches();
    
    match matches.subcommand() {
        Some(("convert", convert_matches)) => {
            let gif_path: &String = convert_matches
                .get_one::<String>("gif-path")
                .expect("`gif-path` is required");
            
            let ani_path: &String = convert_matches
                .get_one::<String>("ani-path")
                .expect("`ani-path` is required");
            
            let frame_rate: &String = convert_matches
                .get_one::<String>("framerate")
                .expect("`framerate` is required");
            
            let hotspot: &String = convert_matches
                .get_one::<String>("hotspot")
                .expect("invalid hotspot");
            
            let result = gif_to_ani(
                gif_path,
                ani_path,
                frame_rate.parse().expect("Failed to parse frame rate"),
                hotspot,
                DebugMode::Default,
            );
            
            match result {
                Err(e) => { println!("{}", e); }
                Ok(_o) => { println!("Conversion successful.") }
            }
        }
        _ => {
            println!("Invalid subcommand.");
        }
    }
    
    Ok(())
}