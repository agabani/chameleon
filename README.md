# Chameleon

## Getting Started

Install dependencies:

```terminal
rustup target add wasm32-unknown-unknown
cargo install --locked sqlx-cli trunk
```

Provision database:

```terminal
sqlx database setup --source .\crates\chameleon-backend\migrations\
```

Run backend:

```terminal
cargo run --bin chameleon-backend
```

Run frontend:

```terminal
trunk serve
```
