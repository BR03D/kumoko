use std::{fs, io::Write, error::Error};
use crate::events::Map;

pub async fn get_map() -> Result<Map, Box<dyn Error>> {

    let data = fs::read("assets/kek.map")?;

    let config = bincode::config::standard();
    let (map, _) = bincode::decode_from_slice(&data, config)?;

    Ok(map)
}

pub async fn save_map(map: Map) -> Result<(), Box<dyn Error>> {
    if map.len == 0 {return Ok(())}

    let config = bincode::config::standard();
    let bin = bincode::encode_to_vec(map, config)?;

    let mut file = fs::File::create("assets/kek.map")?;
    file.write(&bin)?;

    Ok(())
}