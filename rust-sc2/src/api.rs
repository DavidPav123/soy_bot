//! Contains nice wrapper around SC2 API.

use crate::{
    bot::{Locked, Rl},
    client::{SC2Result, WS},
};
use protobuf::Message;
use sc2_proto::sc2api::{Request, Response};
use tungstenite::Message::Binary;

/// SC2 API. Can be accessed through [`self.api()`](crate::bot::Bot::api).
pub struct API(Rl<WS>);
impl API {
    pub(crate) fn new(ws: WS) -> API {
        API(Rl::new(ws))
    }

    /// Sends request and returns a response.
    pub fn send(&self, req: Request) -> SC2Result<Response> {
        let mut ws = self.0.write_lock();

        ws.send(Binary(req.write_to_bytes()?.into()))?;

        let msg = ws.read()?;

        let mut res = Response::new();
        res.merge_from_bytes(msg.into_data().as_ref())?;
        Ok(res)
    }

    /// Sends request, waits for the response, but ignores it (useful when response is empty).
    pub fn send_request(&self, req: Request) -> SC2Result<()> {
        let mut ws = self.0.write_lock();
        ws.send(Binary(req.write_to_bytes()?.into()))?;
        let _ = ws.read()?;
        Ok(())
    }

    /// Sends request, but doesn't wait for the response (use only when more control required,
    /// in common cases prefered to use [`send`] or [`send_request`]).
    ///
    /// [`send`]: Self::send
    /// [`send_request`]: Self::send_request
    pub fn send_only(&self, req: Request) -> SC2Result<()> {
        self.0
            .write_lock()
            .send(Binary(req.write_to_bytes()?.into()))?;
        Ok(())
    }
    /// Waits for a response (useful only after [`send_only`]).
    ///
    /// [`send_only`]: Self::send_only
    pub fn wait_response(&self) -> SC2Result<Response> {
        let msg = self.0.write_lock().read()?;

        let mut res = Response::new();
        res.merge_from_bytes(msg.into_data().as_ref())?;
        Ok(res)
    }
}
