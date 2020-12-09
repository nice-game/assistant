# assistant

## Install diesel, wasm-pack, and client dependencies

    cargo install diesel_cli --no-default-features --features postgres
    cargo install wasm-pack
    cd client
    npm i

## Init DB

Set `DATABASE_URL=postgres://username:password@localhost/diesel_demo`. (This can be done with a .env file right next to this readme.)

    diesel setup
    diesel migration run

## Run Server

    cargo run

## Run Client

    cd client
    npm run dev

## Open in browser

Both the server and client must be running. Go to localhost:8080.

## Regenerate schema.graphql

    graphql-client introspect-schema --output client/schema.json http://localhost:8000/graphql
