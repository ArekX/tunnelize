use crate::common::connection::ConnectionStream;
use crate::create_channel_enum;

use crate::common::channel::OkResponse;

create_channel_enum!(TcpChannelRequest -> TcpChannelResponse, {
    ClientConnect -> OkResponse
});

#[derive(Debug)]
pub struct ClientConnect {
    pub stream: Option<ConnectionStream>,
    pub port: u16,
}