use crate::common::connection::ConnectionStream;
use crate::create_channel_enum;

use crate::common::channel::OkResponse;

create_channel_enum!(TcpChannelRequest -> TcpChannelResponse, {
    ClientConnect -> OkResponse
});

pub struct ClientConnect {
    pub stream: ConnectionStream,
    pub port: u16,
}
