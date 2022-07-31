use bevy::prelude::*;
use kumoko::{Decode, Encode, bevy::{KumokoPlugin, ServerIpAddr}};

fn main() {
    App::new()
    .add_plugin(KumokoPlugin::<Request, Response>::new())
    .insert_resource(ServerIpAddr("[::1]:50052"))
    .run();
}

#[derive(Debug, Clone, Decode, Encode)]
struct Request{
    num: i32
}

#[derive(Debug, Clone, Decode, Encode)]
struct Response{
    num: i32
}
