# ワークスペースの作成

本ブートローダ・システムを実現するためには、ブートローダ、アプリケーションの2つのプロジェクトが必要となってくる。
Cargoのワークスペースの機能をつかい、2つのプロジェクトを1つのワークスペースで管理する。

https://doc.rust-jp.rs/book-ja/ch14-03-cargo-workspaces.html

```
$ mkdir boot-k
$ cd boot-k
$ code .
```

ルートの`Cargo.toml`の`[workspace]` => `members` に子プロジェクトを指定する。edition 2021 を使うために `resolver = "2"` も指定しておく。


```Cargo.toml
[workspace]
members = [
    "bootloader",
    "app-blinky",
]

resolver = "2"
```

その後、コマンドラインで次を実行すれば `bootloader`, `app-blinky` という子プロジェクトが生成される。

この時点で `cargo build` や `cargo clippy` などが双方のプロジェクトに対して実行できる。`cargo run` はどちらのプロジェクトを実行すればよいのかを、ワークスペースの `Cargo.toml` の `default-run` で指定しなければならない。

```
❯ cargo new --bin bootloader
warning: compiling this new package may not work due to invalid workspace configuration

failed to load manifest for workspace member `/.../boot-k/app-blinky`

Caused by:
  failed to read `/.../boot-k/app-blinky/Cargo.toml`

Caused by:
  No such file or directory (os error 2)
     Created binary (application) `bootloader` package

❯ cargo new --bin app-blinky

     Created binary (application) `app-blinky` package

❯ cargo build               
   Compiling app-blinky v0.1.0 (/.../boot-k/app-blinky)
   Compiling bootloader v0.1.0 (/.../boot-k/bootloader)
    Finished dev [unoptimized + debuginfo] target(s) in 0.52s

❯ cargo test 
   Compiling bootloader v0.1.0 (/.../boot-k/bootloader)
   Compiling app-blinky v0.1.0 (/.../boot-k/app-blinky)
    Finished test [unoptimized + debuginfo] target(s) in 0.08s
     Running unittests src/main.rs (target/debug/deps/app_blinky-72fb7b958e84668f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/bootloader-2419ad1a1251e783)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

❯ cargo clippy
    Checking bootloader v0.1.0 (/.../boot-k/bootloader)
    Checking app-blinky v0.1.0 (/.../boot-k/app-blinky)
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
```

