use worker::*;

use muruchat::{pki::{Signature, PublicKey}, handshake::Challenge};

use futures_util::stream::StreamExt;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[durable_object]
pub struct Inbox {
    counter: usize,

    // used for durable object
    #[allow(dead_code)]
    state: State,
    #[allow(dead_code)]
    env: Env,
}

#[durable_object]
impl DurableObject for Inbox {
    fn new(state: State, env: Env) -> Self {
        Self {
            counter: 0,
            state,
            env,
        }
    }

    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        self.counter += 1;

        Response::ok(&format!("counting: {}", self.counter))
    }
}

#[derive(Debug)]
enum FSM {
    WaitingForPK,
    WaitingForSignature(PublicKey, Challenge),
    Authed(PublicKey),
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();

    let router = Router::new();

    router
        .get_async("/chat", |req, _ctx| async move {
            // ensure websocket
            if req.headers().get("Upgrade")? != Some("websocket".to_string()) {    
                return Response::error("Expected Upgrade: websocket", 426);
            }

            // accept connection
            let web_socker_pair = WebSocketPair::new()?;
            web_socker_pair.server.accept()?;

            // process messages async
            wasm_bindgen_futures::spawn_local(async move {
                // start finite state machine to track handshake
                let mut fsm = FSM::WaitingForPK;

                // open stream
                let mut event_stream = web_socker_pair.server.events().expect("could not open stream");

                while let Some(event) = event_stream.next().await {
                    match event.expect("received error in websocket") {
                        WebsocketEvent::Message(msg) => {
                            if let Some(bytes) = msg.bytes() {
                                let next = match fsm {
                                    FSM::WaitingForPK => {
                                        // read public key from client and respond with a challenge.
                                        match PublicKey::from_bytes(&bytes) {
                                            Ok(pk) => {
                                                let challenge = Challenge::new();

                                                if web_socker_pair.server.send_with_bytes(&challenge.bytes()).is_err() {
                                                    break;
                                                }

                                                FSM::WaitingForSignature(pk, challenge)
                                            },
                                            _ => break,
                                        }
                                    }
                                    FSM::WaitingForSignature(pk, challenge) => {
                                        // verify the signature against challenge
                                        match Signature::from_bytes(&bytes) {
                                            Ok(sig) => {
                                                if !challenge.verify(&pk, &sig) {
                                                    break;
                                                }

                                                FSM::Authed(pk)
                                            },
                                            _ => break,
                                        }
                                    },
                                    FSM::Authed(pk) => {
                                        // run echo server in a loop
                                        if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                                            let res = format!("Hello, {}!", s);

                                            if web_socker_pair.server.send_with_bytes(&res.as_bytes()).is_err() {
                                                break;
                                            }
                                        }

                                        FSM::Authed(pk)
                                    }
                                };

                                fsm = next;
                            }
                        },
                        WebsocketEvent::Close(_event) => break,
                    }
                }
            });

            Response::from_websocket(web_socker_pair.client)
        })
        .on_async("/counter", |_req, ctx| async move {
            let namespace = ctx.durable_object("INBOX")?;
            let stub = namespace.id_from_name("A")?.get_stub()?;
            stub.fetch_with_str("/messages").await
        })
        .run(req, env)
        .await
}
