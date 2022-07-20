use std::{fs, io::Write, error::Error};
use crate::events::Map;

pub async fn get_map() -> Result<Map, Box<dyn Error>> {

    let data = fs::read("assets/kek.map")?;
    let map = bincode::deserialize(&data)?;

    Ok(map)
}

pub async fn save_map(map: Map) -> Result<(), Box<dyn Error>> {
    if map.len == 0 {return Ok(())}
    let mut file = fs::File::create("assets/kek.map")?;
    file.write(&bincode::serialize(&map)?)?;

    Ok(())
}