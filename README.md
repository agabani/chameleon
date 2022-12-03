# Chameleon

## Getting Started

Install dependencies:

```terminal
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

Run frontend:

```terminal
trunk serve .\crates\chameleon-frontend\index.html
```

Run backend:

```terminal
cargo run --bin chameleon-backend
```
