# Secret
A simple program to hide one image in another by replacing pixel bits

## Build
```
cargo build --release
```

## Usage
```
./secret.exe hide source.jpg secret.jpg image.png
```
```
./secret.exe decrypt image.png decrypted.png
```
Or when you're testing an application
```
cargo run hide source.jpg secret.jpg image.png
```
```
cargo run decrypt image.png decrypted.png
```