//! Kumoko provides a built-in bevy-plugin.

use std::marker::PhantomData;
use kumoko::{client::{Client, Receiver, Sender}, Message};
use bevy::prelude::*;

#[derive(Debug, Deref)]
pub struct ServerIpAddr(pub &'static str);

pub trait Msg: Message + Sync + Clone{}
impl<T> Msg for T where T: Message + Sync + Clone{}

pub struct KumokoPlugin<Request: Msg, Response: Msg>{
    req: PhantomData<Request>,
    res: PhantomData<Response>,
}

impl<Request: Msg, Response: Msg> Plugin for KumokoPlugin<Request, Response>{
    fn build(&self, app: &mut App) {
        app
            .insert_resource(tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
            )
            .add_event::<Request>()
            .add_event::<Response>()
            .add_startup_system(Self::init)
            .add_system(Self::send)
            .add_system(Self::receive)
        ;
    }
}

impl<Request: Msg, Response: Msg> KumokoPlugin<Request, Response> {
    pub fn new() -> Self{
        Self { req: PhantomData, res: PhantomData }
    }

    fn init(
        runtime: Res<tokio::runtime::Runtime>,
        mut commands: Commands,
        ip: Res<ServerIpAddr>
    ) {
        let ip = **ip;
        // this blocks the main thread for now
        let client = runtime.block_on(runtime.spawn(async move{
            Client::<Request, Response>::connect(ip).await.expect("We couldnt connect to the Server! Oh no")
        })).unwrap();

        let (rec, send) = client.into_split();
        commands.insert_resource(send);
        commands.insert_resource(rec);
    }

    fn send(
        send: ResMut<Sender<Request>>,
        mut event: EventReader<Request>,
    ) {
        let send = send.into_inner();

        for e in event.iter(){
            // this clones the message...for no reason. kinda cringe ngl.
            send.try_send(e.clone());
        }
    }

    fn receive(
        mut rec: ResMut<Receiver<Response>>,
        mut event: EventWriter<Response>,
    ) {
        let resp = match rec.try_get_response(){
            Ok(resp) => resp,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => panic!("We disconnected! Oh no!")
        };

        event.send(resp);
    }
}