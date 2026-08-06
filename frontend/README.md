rustup target add wasm32-unknown-unknown

cargo install --locked trunk

trunk serve --open

docker build -t frontend .

docker run -d -p 8080:80 --name frontend-app frontend

