deploy target_host:
    cross build --release --target "aarch64-unknown-linux-gnu"
    curl -X POST "http://{{target_host}}:3000/api/update/self" \
         -H "Content-Type: application/octet-stream" \
         --data-binary "@target/aarch64-unknown-linux-gnu/release/control-app"

[parallel]
serve: serve-backend serve-frontend

serve-backend:
    cargo run -- serve

[working-directory: "control-app/frontend"]
serve-frontend:
    npm run dev
