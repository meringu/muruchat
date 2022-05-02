use serde_json::json;
use worker::*;

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

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", |_, _| Response::ok("Hello from Workers!"))
        .get_async("/foo", |req, ctx| async move {
            match foo(req, ctx).await {
                Ok(s) => Response::ok(s),
                Err(e) => Response::error(e.to_string(), 500),
            }
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
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
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
