use std::{fs, io::Write};
use crate::{events::Map, MyError};

pub async fn get_map() -> Result<Map, MyError> {

    let data = fs::read("assets/kek.map")?;
    let map = bincode::deserialize(&data)?;

    Ok(map)
}

pub async fn save_map(map: Map) -> Result<(), MyError> {
    if map.len == 0 {return Ok(())}
    let mut file = fs::File::create("assets/kek.map")?;
    file.write(&bincode::serialize(&map)?)?;

    Ok(())
}