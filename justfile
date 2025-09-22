deploy target_host:
    cross build --release --target "aarch64-unknown-linux-gnu"
    curl -X POST "http://{{target_host}}:3000/api/update/self" \
         -H "Content-Type: application/octet-stream" \
         --data-binary "@target/aarch64-unknown-linux-gnu/release/control-app"

[parallel]
serve: serve-backend serve-frontend

serve-backend:
    cargo run -- serve

[working-directory: "software/frontend"]
serve-frontend:
    npm run dev

export PATH := "~/.cargo/bin:" + env_var('PATH')

[working-directory: "software/firmware"]
build-firmware:
    . "$HOME/.rustup/export-esp.sh"
    cargo build --release

[working-directory: "software/firmware"]
flash:
    . "$HOME/.rustup/export-esp.sh"
    cargo build --release
    echo "$(pwd)"
    espflash flash --chip esp32 --port /dev/ttyUSB0 \
        "../../target/xtensa-esp32-none-elf/release/firmware"
