use kumoko::{Decode, Encode};

#[derive(Debug, Clone, Decode, Encode)]
pub enum Request{
    SaveMap(Map), 
    MapRequest,
}

#[derive(Debug, Clone, Decode, Encode)]
pub enum Response {
    SendMap(Map),
    Okie,
}

#[derive(Clone, Decode, Encode)]
pub struct Map{
    pub h: Vec<Vec<u8>>,
    pub t: Vec<Vec<Terrain>>,
    pub len: usize,
}

impl std::fmt::Debug for Map{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Map").field("len", &self.len).finish()
    }
}

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq)]
pub enum Terrain{
    Grass,
    Sand,
    Water,
}
