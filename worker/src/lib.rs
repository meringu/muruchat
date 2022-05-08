use serde_json::json;
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
pub struct Chat {
    counter: usize,
    state: State,
    env: Env,
}

#[durable_object]
impl DurableObject for Chat {
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
        .get_async("/chat", |req, ctx| async move {
            console_log!("chat started");
            // let chat_id = ctx.param("chat_id").unwrap();

            // ensure websocket
            if req.headers().get("Upgrade")? != Some("websocket".to_string()) {    
                return Response::error("Expected Upgrade: websocket", 426);
            }

            console_log!("upgrade ok");

            // accept connection
            let web_socker_pair = WebSocketPair::new()?;
            web_socker_pair.server.accept()?;

            wasm_bindgen_futures::spawn_local(async move {
                let mut fsm = FSM::WaitingForPK;

                let mut event_stream = web_socker_pair.server.events().expect("could not open stream");

                while let Some(event) = event_stream.next().await {
                    match event.expect("received error in websocket") {
                        WebsocketEvent::Message(msg) => {
                            if let Some(bytes) = msg.bytes() {
                                let next = match fsm {
                                    FSM::WaitingForPK => {
                                        match PublicKey::from_bytes(&bytes) {
                                            Ok(pk) => {
                                                let challenge = Challenge::new();

                                                if let Err(_) = web_socker_pair.server.send_with_bytes(&challenge.bytes()) {
                                                    break;
                                                }

                                                FSM::WaitingForSignature(pk, challenge)
                                            },
                                            _ => break,
                                        }
                                    }
                                    FSM::WaitingForSignature(pk, challenge) => {
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
                                        if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                                            let res = format!("Hello, {}!", s);

                                            if let Err(_) = web_socker_pair.server.send_with_bytes(&res.as_bytes()) {
                                                break;
                                            }
                                        }
                                        FSM::Authed(pk)
                                    }
                                };

                                fsm = next;
                            }
                        },
                        WebsocketEvent::Close(event) => break,
                    }
                }
            });

            Response::from_websocket(web_socker_pair.client)
        })
        .on_async("/foo", |req, ctx| async move {
            match foo(req, ctx).await {
                Ok(s) => Response::ok(s),
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .on_async("/counter", |_req, ctx| async move {
            let namespace = ctx.durable_object("TEST")?;
            let stub = namespace.id_from_name("A")?.get_stub()?;
            stub.fetch_with_str("/messages").await
        })
        .post_async("/form/:field", |mut req, ctx| async move {
            if let Some(name) = ctx.param("field") {
                let form = req.form_data().await?;
                match form.get(name) {
                    Some(FormEntry::Field(value)) => {
                        return Response::from_json(&json!({ name: value }))
                    }
                    Some(FormEntry::File(_)) => {
                        return Response::error("`field` param in form shouldn't be a File", 422);
                    }
                    None => return Response::error("Bad Request", 400),
                }
            }

            Response::error("Bad Request", 400)
        })
        .run(req, env)
        .await
}

async fn foo(_: worker::Request, ctx: worker::RouteContext<()>) -> Result<String> {
    let store = ctx.kv("chat")?;
    let bytes = match store.get("counter").bytes().await? {
        Some(vec) => {
            let v: Vec<u8> = vec;
            let ob: [u8; 4] = v
                .try_into()
                .map_err(|_| Error::RustError("failed to get counter".to_owned()))?;

            ob
        }
        None => [0; 4],
    };

    let mut counter = u32::from_le_bytes(bytes);
    counter += 1;

    store
        .put_bytes("counter", &counter.to_le_bytes())?
        .expiration_ttl(60)
        .execute()
        .await?;

    let mut buf = [0u8; 16]; // 128 bit
    getrandom::getrandom(&mut buf).map_err(|e| Error::RustError(e.to_string()))?;
    let h = hex::encode(buf);

    return Ok(format!("counter:{}, hex:{}", counter, h));
}
