use std::{fs::File, io::BufReader, path::Path};

use chrono::{DateTime, Utc};
use quick_xml::{Reader, events::Event};

use crate::internal::{io::io_errors::IOError, model::{spatial::points::SpatialPoint, track::common::SpatialTrack}};


/*
    Loads a track from a track a file with @path
 */
pub fn load_track(path : &Path) -> Result<SpatialTrack, IOError> {
    if path.extension().ok_or(IOError::invalid_path(path.to_str().unwrap_or("unkown path"), "Could not collect path extension"))? != "gpx" {
        return Err(IOError::format_not_supported(path.to_str().unwrap(), "Only supports gpx track format"));
    }

    let str_path = path.to_str().unwrap_or("unkown file path");

    const INITIAL_ALLOCATION_SIZE : usize = 12_000;
    const FILE_BUFFER_SIZE : usize = 64 * 1024;
    
    let file = File::open(path).map_err(
        |err| {return IOError::xml_reader(str_path, err.to_string().as_str());}
    )?;
    let reader = BufReader::with_capacity(FILE_BUFFER_SIZE, file);
    let mut reader = Reader::from_reader(reader);

    reader.trim_text(true);
    

    let mut xml_buffer = Vec::with_capacity(INITIAL_ALLOCATION_SIZE);
    let mut points = Vec::with_capacity(INITIAL_ALLOCATION_SIZE);

    let mut initial_time : Option<f64> = None;
    let mut initial_stamp : Option<DateTime<Utc>> = None;
    let mut current_time: f64 = 0.0;
    let mut in_time = false;
    
    let mut in_ele = false;

    let mut lat = 0.0;
    let mut lon = 0.0;
    let mut elevation = 0.0;

    loop {
        match reader.read_event_into(&mut xml_buffer).map_err(
            |err| { IOError::xml_reader(str_path, err.to_string().as_str())}
        )? {
            
            Event::Start(element) if element.name().as_ref() == b"time" => {
                in_time = true;
            }

            Event::Start(element) if element.name().as_ref() == b"ele" => {
                in_ele = true;
            }

            Event::Start(element) if element.name().as_ref() == b"trkpt" => {
                for attribute in element.attributes() {
                    let attribute = attribute.unwrap();
                    let str_attribute = unsafe { std::str::from_utf8_unchecked(&attribute.value.as_ref()) };
                    match attribute.key.as_ref() {
                        b"lat" => lat = str_attribute.parse::<f64>().map_err(
                            |_| IOError::xml_parser(path.to_str().unwrap_or("unkown file path"),format!("Invalid data for f64 conversion in lat field {:?}", str_attribute).as_str())
                        )?,
                        b"lon" => lon = str_attribute.parse::<f64>().map_err(
                            |_| IOError::xml_parser(path.to_str().unwrap_or("unkown file path"),format!("Invalid data for f64 conversion in long field {:?}", str_attribute).as_str())
                        )?,
                        _ => {}
                    }
                }
            }

            Event::Text(element) if in_time && !in_ele => {
                let str_elem = unsafe { std::str::from_utf8_unchecked(element.as_ref()) };


                if let Some(initial_moment) = initial_time {
                    current_time += fast_parse_gpx_seconds(str_elem) - initial_moment;
                } else {
                    let date_time = DateTime::parse_from_rfc3339(str_elem)
                        .map_err(|err| IOError::xml_parser(str_path, err.to_string().as_str()))?
                        .with_timezone(&Utc);

                    initial_stamp = Some(date_time);
                    initial_time = Some(0.0);
                    current_time = 0.0;
                }

                in_time = false;
            }

            Event::Text(e) if in_ele && !in_time => {
                let str_elem = unsafe { std::str::from_utf8_unchecked(e.as_ref()) };
                elevation = str_elem.parse::<f64>().map_err(
                      |err| { IOError::xml_reader(str_path, err.to_string().as_str())}
                )?;
            }

            Event::End(e) if e.name().as_ref() == b"trkpt" => {
                points.push(SpatialPoint { lon, lat, elev: Some(elevation), delta_seconds: Some(current_time) });

                lat =0.0;
                lon =0.0;
                elevation =0.0;
            }

            Event::End(e) if e.name().as_ref() == b"time" => {
                in_time = false;
            }

            Event::End(e) if e.name().as_ref() == b"ele" => {
                in_ele = false;
            }

            Event::Eof => break,
            _ => {}
        }

        xml_buffer.clear();
    }

    Ok(SpatialTrack { 
        track: points, 
        start_time: initial_stamp.unwrap_or_default()
    })

}

#[inline]
fn fast_parse_gpx_seconds(s: &str) -> f64 {
    let byte_buffer = s.as_bytes();

    // HH:MM:SS
    let hour =
        (byte_buffer[11] - b'0') as u32 * 10 + (byte_buffer[12] - b'0') as u32;
    let min =
        (byte_buffer[14] - b'0') as u32 * 10 + (byte_buffer[15] - b'0') as u32;
    let sec =
        (byte_buffer[17] - b'0') as u32 * 10 + (byte_buffer[18] - b'0') as u32;

    let mut frac = 0u32;
    let mut scale = 1_000_000_000u32;

    if byte_buffer[19] == b'.' {
        let mut i = 20;
        while byte_buffer[i] != b'Z' {
            frac = frac * 10 + (byte_buffer[i] - b'0') as u32;
            scale /= 10;
            i += 1;
        }
    }

    (hour * 3600 + min * 60 + sec) as f64
        + (frac as f64 / scale as f64)
}

