# rm -rf ./out/*
# cargo run ./bindings/ ./out/  
# dirlist=$(find ./out/ -mindepth 1 -maxdepth 1 -type f)

# for file in $dirlist
# do
#   ../../github/graindom/wit-bindgen/target/debug/wit-bindgen grain $file --out-dir ./dist/
# done
../../github/graindom/wit-bindgen/target/debug/wit-bindgen grain ./out/ --out-dir ./dist/