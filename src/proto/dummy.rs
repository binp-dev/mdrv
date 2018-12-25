use std::collections::{VecDeque};

use ::channel::{Sender, SinglePoll};
use ::proxy::{self, Control, Eid};
use ::wrapper::{self, UserProxy, UserHandle};

pub use wrapper::{Tx, Rx};

pub struct Proxy {}

impl Proxy {
    fn new() -> Self {
        Self {}
    }
}

impl proxy::Proxy for Proxy {
    fn attach(&mut self, _ctrl: &Control) -> ::Result<()> {
        Ok(())
    }

    fn detach(&mut self, _ctrl: &Control) -> ::Result<()> {
        Ok(())
    }

    fn process(&mut self, _ctrl: &mut Control, _readiness: mio::Ready, _eid: Eid) -> ::Result<()> {
        Ok(())
    }
}

impl UserProxy<Tx, Rx> for Proxy {
    fn set_send_channel(&mut self, _tx: Sender<Rx>) {}
    fn process_recv_channel(&mut self, _ctrl: &mut Control, _msg: Tx) -> ::Result<()> {
        Ok(())
    }
}

pub struct Handle {
    pub msgs: VecDeque<Rx>,
}

impl Handle {
    fn new() -> Self {
        Self {
            msgs: VecDeque::new(),
        }
    }
}

impl UserHandle<Tx, Rx> for Handle {
    fn set_send_channel(&mut self, _tx: Sender<Tx>) {}
    fn process_recv_channel(&mut self, msg: Rx) -> ::Result<()> {
        self.msgs.push_back(msg);
        Ok(())
    }
}

pub fn create() -> ::Result<(wrapper::Proxy<Proxy, Tx, Rx>, wrapper::Handle<Handle, Tx, Rx>)> {
    wrapper::create(Proxy::new(), Handle::new())
}

pub fn wait_msgs(h: &mut wrapper::Handle<Handle, Tx, Rx>, sp: &mut SinglePoll, n: usize) -> ::Result<()> {
    let ns = h.user.msgs.len();
    loop {
        if let Err(e) = sp.wait(None) {
            break Err(::Error::Channel(e.into()));
        }
        if let Err(e) = h.process() {
            break Err(e);
        }
        if h.user.msgs.len() - ns >= n {
            break Ok(());
        }
    }
}

pub fn wait_close(h: &mut wrapper::Handle<Handle, Tx, Rx>, sp: &mut SinglePoll) -> ::Result<()> {
    loop {
        if let Err(e) = sp.wait(None) {
            break Err(::Error::Channel(e.into()));
        }
        match h.process() {
            Ok(()) => continue,
            Err(err) => match err {
                ::Error::Proxy(proxy::Error::Closed) => break Ok(()),
                other => break Err(other),
            }
        }
    }
}