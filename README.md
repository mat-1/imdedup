```sh
RUSTFLAGS='-C target-cpu=native' cargo b -r
sudo cp target/release/imdedup /usr/bin
imdedup ~/pictures/cats/sandcats # --delete
```
