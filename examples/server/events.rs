use project_g::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Request{
    SaveMap(Map), 
    MapRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Response {
    SendMap(Map),
    Okie,
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Terrain{
    Grass,
    Sand,
    Water,
}
