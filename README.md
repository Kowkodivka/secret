# Secret
A simple program to hide one image in another by replacing pixel bits

## Build
```
cargo build --release
```

## Usage
```
./secret.exe hide_img --source source.jpg --secret secret.jpg --output image.png
```
```
./secret.exe decrypt_img --source image.png --output decrypted.png
```
Or when you're testing an application
```
cargo run hide_img --source source.jpg --secret secret.jpg --output image.png
```
```
cargo run decrypt_img --source image.png --output decrypted.png
```