use crate::player::PlayerId;
pub mod messages;

#[derive(Debug, Clone)]
pub enum ServerEvent {
    NoEvent,
    ClientConnected(PlayerId),
    ClientDisconnected(PlayerId),
    ClientMessage(PlayerId, messages::ToServer),
}

#[derive(Debug, Clone)]
pub enum ClientEvent {
    NoEvent,
    Connected,
    Disconnected,
    ServerMessage(messages::ToClient),
}

pub trait Server {
    fn receive_event(&mut self) -> ServerEvent;

    fn send(&mut self, client: PlayerId, message: messages::ToClient);
}

pub trait Client {
    fn receive_event(&mut self) -> ClientEvent;
    fn send(&mut self, _: messages::ToServer);
}

pub mod dummy;