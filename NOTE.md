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

# ハードウエアの準備

2枚のRP2040ボードを使用する。一枚は `picoprobe.uf2` のファームウエアを書き込んでデバッガとして使い、もう一枚はターゲットとして使う。ターゲットには、一枚目のデバッガから SWD 接続する。

ハードウエアの構成や、関連ツールのインストールについては別エントリー。

https://nkon.github.io/RasPico-Rust/

# テンプレート・プロジェクトの実行

https://github.com/rp-rs/rp2040-project-template

からプロジェクトテンプレートをコピーしてくる。

```
❯ git clone https://github.com/rp-rs/rp2040-project-template
Cloning into 'rp2040-project-template'...
remote: Enumerating objects: 391, done.
remote: Counting objects: 100% (210/210), done.
remote: Compressing objects: 100% (92/92), done.
remote: Total 391 (delta 148), reused 140 (delta 116), pack-reused 181
Receiving objects: 100% (391/391), 82.32 KiB | 2.42 MiB/s, done.
Resolving deltas: 100% (195/195), done.
```
ここで、ワークスペースの Cargo.toml => members に rp2040-project-template を付け加える。

`cargo build`でビルド。

```
❯ cd rp2040-project-template/ 

❯ cargo build
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /.../boot-k/rp2040-project-template/Cargo.toml
workspace: /.../boot-k/Cargo.toml
    Updating crates.io index
  Downloaded proc-macro2 v1.0.70
  Downloaded rp2040-hal v0.9.1
  Downloaded 2 crates (222.5 KB) in 0.44s
   Compiling proc-macro2 v1.0.70
   Compiling unicode-ident v1.0.12
   Compiling syn v1.0.109
...
   Compiling rp2040-hal-macros v0.1.0
   Compiling pio v0.2.1
   Compiling rp2040-hal v0.9.1
    Finished dev [unoptimized + debuginfo] target(s) in 10.55s
```

`cargo run` で `probe-rs`がファームウェアをデバイスに転送し、実行する。

LEDが点滅し、RTTで実行端末にメッセージが表示される。

```
❯ cargo run  
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /.../boot-k/rp2040-project-template/Cargo.toml
workspace: /.../boot-k/Cargo.toml
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `probe-rs run --chip RP2040 --protocol swd /.../boot-k/target/thumbv6m-none-eabi/debug/rp2040-project-template`
     Erasing sectors ✔ [00:00:00] [] 52.00 KiB/52.00 KiB @ 64.62 KiB/s (eta 0s )
 Programming pages   ✔ [00:00:01] [] 52.00 KiB/52.00 KiB @ 30.45 KiB/s (eta 0s )    Finished in 2.544s
INFO  Program start
└─ rp2040_project_template::__cortex_m_rt_main @ src/main.rs:27  
INFO  on!
└─ rp2040_project_template::__cortex_m_rt_main @ src/main.rs:64  
INFO  off!
└─ rp2040_project_template::__cortex_m_rt_main @ src/main.rs:67  
INFO  on!
└─ rp2040_project_template::__cortex_m_rt_main @ src/main.rs:64  
INFO  off!
└─ rp2040_project_template::__cortex_m_rt_main @ src/main.rs:67  
```
