cp src/index.html hit.mp3 target/wasm32-unknown-emscripten/debug/
cd target/wasm32-unknown-emscripten/debug/
echo  "GO TO http://127.0.0.1:8000"
echo
python3 -m http.server 8000
