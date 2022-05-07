# MuruChat

TODO: description of what and how

## Local development

### Worker

**Prerequisites**

- [Wrangler](https://developers.cloudflare.com/workers/cli-wrangler/install-update/)

**Running**

```
wranlger login
wrangler dev
```

### Web

**Prerequisites**

- [Rust](https://rustup.rs/)
- [Trunk](https://trunkrs.dev/)

Add the Rust wasm target

```
rustup target add wasm32-unknown-unknown
```

**Running**

```
trunk serve
```

Open [http://localhost:8080](http://localhost:8080)
