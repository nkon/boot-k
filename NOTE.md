- [ワークスペースの作成](#ワークスペースの作成)
- [ハードウエアの準備](#ハードウエアの準備)
- [テンプレート・プロジェクトの実行](#テンプレートプロジェクトの実行)
- [テンプレート・プロジェクトの解説](#テンプレートプロジェクトの解説)
  - [RP2040の基礎](#rp2040の基礎)
    - [MCUコア](#mcuコア)
    - [内蔵ROM](#内蔵rom)
      - [起動の流れ](#起動の流れ)
    - [QSPI Flash](#qspi-flash)
    - [メモリ・マップ](#メモリマップ)
  - [テンプレート・プロジェクトに組み込まれているもの](#テンプレートプロジェクトに組み込まれているもの)
    - [defmt-rtt](#defmt-rtt)
    - [flip-link](#flip-link)
    - [rp-pico(BSP), rp2040-hal(HAL), rp2040-pac(PAC), cortex-m(MAC)](#rp-picobsp-rp2040-halhal-rp2040-pacpac-cortex-mmac)
    - [rp2040-boot2](#rp2040-boot2)
    - [cortex-m-rt](#cortex-m-rt)
      - [リンカ・スクリプト](#リンカスクリプト)
    - [panic-probe](#panic-probe)
    - [VS Code debugger](#vs-code-debugger)
- [bootloader プロジェクトの作成](#bootloader-プロジェクトの作成)


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

からプロジェクトテンプレートをクローンしてくる。

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

# テンプレート・プロジェクトの解説

テンプレート・プロジェクトを使えば、非常に簡単に Lチカが実現できて良いが、その裏側でなにが起こっているのかをきちんと把握して置かなければ、低レイヤでの改造がうまくいかなくなる。

## RP2040の基礎

今回のターゲットボードである Raspberry Pi Pico は、RP2040というチップをコアに、QSPI フラッシュメモリ(W25Q16JV)、LED、スイッチ(BOOTSEL)、USBインターフェイス、電源などを搭載したもの。

まずは RP2040 というMCUの構成を知っておかなければならない。

https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf

以後の説明で必要になるところだけ。

### MCUコア

RP2040はCortex-M0+のコアを2つ持つ

ペリフェラル・バスとしてAPBがあり、通常のペリフェラルがぶら下がっている。

高速バスとして。AHB-Liteがあり、内蔵メモリ、Flash XIPコントローラ、PIO、USBはAHB-Liteに繋がっている。

### 内蔵ROM

RP2040は内蔵のプログラムメモリとして16KBのROMがある。ROMにはブートローダが格納されている。ユーザプログラムは外付けのQSPIフラッシュに格納され、XIPで実行される。

通常、外付けのQSPIフラッシュのコードの一部として rp2040-boot2 が組み込まれており、内蔵ROM→boot2→ユーザコードの順に実行される。

* Initial startup routine: スタートアップ・ルーチン
* Flash boot sequence: QSPIフラッシュから起動するための仕組み
* Flash programming routines: QSPIフラッシュに書き込むためのサブルーチン
* USB mass storage device with UF2 support: USBマスストレージを提供しUF2ファームウエアからブートする
* Utility libraries such as fast floating point: ユーティリティ関数を提供(浮動小数点演算など)

#### 起動の流れ

内蔵ROMからの起動手順はRP2040のデータシートの 2.8 Figure 15に詳しく書かれている。
概略は次のとおり。

* 起動時はCore0のみ起動。Core1はスリープ。
* 起動時にQSPI_CSをチェック。QSPI_CS は、RasPicoボードでは、その名のとおりQSPI Flash の#CS端子と、BOOTSELスイッチに接続されている。
    * QSPI_CS がH(BOOTSELが押されていない) => QSPIからブート
        + 256バイトをロード => そこには Flash Second Stage(boot2)が格納されているので。それを実行。
    * QSPI_CS がL(BOOTSELが押されている) => USB・マス・ストレージモードでブート
        + QSPI フラッシュを USBドライブとしてPCに接続し、USBドライブにファームウエアが書かれるのを待つ

### QSPI Flash

上述のように RP2040 はアプリケーション・コードのための内蔵メモリを持たない。外部にQSPI Flashを接続し、そこにアプリケーション・コードを格納し、XIP(Execute In Place)で実行する。そのために、外付けQSPI Flashは十分な速度のQSPI バスで接続されなければならないし、キャッシュも内蔵している。

内蔵ROMのブートローダ => QSPI Flashに内蔵された second stage bootloader(boot2) => アプリケーション・コード　の順に実行される。

boot2は、QSPI Flashの先頭部分に格納される。

boot2の行わなければならないことは次のとおり。

* SSI(Synchronous Serial Interface) を適切に設定する。
* QSPI Flashのチップを適切に設定する。Quad モードで動作するようにする、など。
* 呼び出すアプリケーションのために、割り込みベクタ、(スタートアップルーチンの)開始アドレス、スタックポインタなどを設定する。
* boot2 のトータルの容量は256byte。その中の最後4バイトはCRCチェックサム。

### メモリ・マップ

データシート 2.2 Address Map より

| Address   | size           | Physical                      |           |             |     |  Address  | Alias                   |
|-----------|----------------|-------------------------------|-----------|-------------|-----|-----------|-------------------------|
|0x0000_0000|  16K(    0x400)| Internal ROM                  |           |             |     |0x0000_0000|ROM_BASE                 |
|0x1000_0000|2048K(0x20_0000)| QSPI Flash(XIP)               |           |             |     |0x1000_0000|XIP_BASE                 |
|           |                |                               |           |             |     |0x1100_0000|XIP_NOALLOC_BASE         |
|           |                |                               |           |             |     |0x1200_0000|XIP_NOCACHE_BASE         |
|           |                |                               |           |             |     |0x1300_0000|XIP_NOCACHE_NOALLOCB_BASE|
|           |                |                               |           |             |     |0x1400_0000|XIP_CTRL_BASE            |
|           |                |                               |           |             |     |0x1500_0000|XIP_SRAM_BASE            |
|           |                |                               |           |             |     |0x1500_0400|XIP_SRAM_END             |
|           |                |                               |           |             |     |0x1800_0000|XIP_SSI_BASE             |
|0x2000_0000| 256K( 0x4_0000)| SRAM                          |0x2000_0000|64K(0x1_0000)|SRAM0|0x2000_0000|SRAM_BASE                |
|           |                |                               |           |             |     |0x2000_0000|SRAM_STRIPED_BASE        |
|           |                |                               |           |             |     |0x2000_0000|SRAM0_BASE               |
|           |                |                               |0x2001_0000|64K(0x1_0000)|SRAM1|0x2001_0000|SRAM1_BASE               |
|           |                |                               |0x2002_0000|64K(0x1_0000)|SRAM2|0x2002_0000|SRAM2_BASE               |
|           |                |                               |0x2003_0000|64K(0x1_0000)|SRAM3|0x2003_0000|SRAM3_BASE               |
|           |                |                               |           |             |     |0x2004_0000|SRAM_STRIPED_END         |
|           |                |                               |0x2004_0000| 4K(  0x1000)|SRAM4|0x2004_0000|SRAM4_BASE               |
|           |                |                               |0x2004_1000| 4K(  0x1000)|SRAM5|0x2004_1000|SRAM5_BASE               |
|           |                |                               |           |             |     |0x2004_2000|SRAM_END                 |
|0x4000_0000|                | APB Peripherals               |           |             |     |0x4000_0000|SYSINFO_BASE             |
|           |                |                               |           |             |     |0x4000_4000|SYSCFG_BASE              |
|           |                |                               |           |             |     |0x4000_8000|CLOCKS_BASE              |
|           |                |                               |           |             |     |0x4000_c000|RESETS_BASE              |
|           |                |                               |           |             |     |0x4001_0000|PSM_BASE                 |
|           |                |                               |           |             |     |0x4001_4000|IO_BANK0_BASE            |
|           |                |                               |           |             |     |0x4001_8000|IO_QSPI_BASE             |
|           |                |                               |           |             |     |0x4001_c000|PADS_BANK0_BASE          |
|           |                |                               |           |             |     |0x4002_0000|PADS_QSPI_BASE           |
|           |                |                               |           |             |     |0x4002_4000|XOSC_BASE                |
|           |                |                               |           |             |     |0x4002_8000|PLL_SYS_BASE             |
|           |                |                               |           |             |     |0x4002_c000|PLL_USB_BASE             |
|           |                |                               |           |             |     |0x4003_0000|BUSCTRL_BASE             |
|           |                |                               |           |             |     |0x4003_4000|UART0_BASE               |
|           |                |                               |           |             |     |0x4003_8000|UART1_BASE               |
|           |                |                               |           |             |     |0x4003_c000|SPI0_BASE                |
|           |                |                               |           |             |     |0x4004_0000|SPI1_BASE                |
|           |                |                               |           |             |     |0x4004_4000|I2C0_BASE                |
|           |                |                               |           |             |     |0x4004_8000|I2C1_BASE                |
|           |                |                               |           |             |     |0x4004_c000|ADC_BASE                 |
|           |                |                               |           |             |     |0x4005_0000|PWM_BASE                 |
|           |                |                               |           |             |     |0x4005_4000|TIMER_BASE               |
|           |                |                               |           |             |     |0x4005_8000|WATCHDOG_BASE            |
|           |                |                               |           |             |     |0x4005_c000|RTC_BASE                 |
|           |                |                               |           |             |     |0x4006_0000|RTC_BASE                 |
|           |                |                               |           |             |     |0x4006_4000|VREG_AND_CHIP_RESET_BASE |
|           |                |                               |           |             |     |0x4006_c000|TBMAN_BASE               |
|0x5000_0000|                | AHB-Lite Peripherals          |           |             |     |0x5000_0000|DMA_BASE                 |
|           |                |                               |           |             |     |0x5010_0000|USBCTRL_BASE             |
|           |                |                               |           |             |     |0x5010_0000|USBCTRL_DPRAM_BASE       |
|           |                |                               |           |             |     |0x5011_0000|USBCTRL_REGS_BASE        |
|           |                |                               |           |             |     |0x5020_0000|PIO0_BASE                |
|           |                |                               |           |             |     |0x5030_0000|PIO1_BASE                |
|           |                |                               |           |             |     |0x5040_0000|XIP_AUX_BASE             |
|0xd000_0000|                | IOPORT Registers              |           |             |     |0xd000_0000|SIO_BASE                 |
|0xe000_0000|                | Cortex-M0+ internal registers |           |             |     |0xe000_0000|PPB_BASE                 |

## テンプレート・プロジェクトに組み込まれているもの

テンプレート・プロジェクトを使えば、非常に簡単に Lチカが実現できて良いが、その裏側でなにが起こっているのかをきちんと把握して置かなければ、低レイヤでの改造がうまくいかなくなる。

### defmt-rtt

defmt はデバッグプリントライブラリ。出力先を柔軟に制御でき、次のデバッグ出力が可能。またシェル変数でデバッグレベルを指定することで出力の多寡を調整できる。

```
defmt::trace!("trace");
defmt::debug!("debug");
defmt::info!("info");
defmt::warn!("warn");
defmt::error!("error");
```

このテンプレートでは `defmt-rtt` を使って RTT 経由でメッセージが出力される。RTT(Real Time Transfer) は SWD(JTAG) の上にUART的なデータを流す技術。

`defmt` は単にクレートを use するだけでなく `.cargo/config.toml` に
`rustflags = "-C", "link-arg=-Tdefmt.x"` とリンカスクリプトの指定が必要なことに注意。

### flip-link

このテンプレートは `flip-link` を使ってスタックオーバーフローをしないようにしている。

通常はRAMアドレスの低位側にBSS、ヒープが配置され、RAMアドレスの最高位側からスタックが消費されていく。この場合、スタック・オーバーフローすればヒープ領域が破壊される。`flip-link`の場合は、RAMアドレスの高位側にBSSとヒープが予め配置され、その下からスタックが消費されていく。そうすれば、スタック・オーバーフローした場合はRAMの最低位アドレスに到達するだけでヒープが壊れることはない。

### rp-pico(BSP), rp2040-hal(HAL), rp2040-pac(PAC), cortex-m(MAC)

このテンプレートは Rasberry Pi Pico のBSP(Board Support Package)を使っている。

RP2040がチップの名前、それを使ったボードが Raspberry Pi Picoだ。ボード上には RP2040、W25Q16JV QSPI Flashメモリ、BOOTSEL スイッチ、USBインターフェイス、GPIO インターフェイス、LEDなどが搭載されている。

Embedded Rustでデファクトである rust-embedded プロジェクトのアーキテクチャでは、低レイヤ側から Micro architecture crete(MAC) がコアそのものをサポートし、PAC(Peripheral Access Crate)がペリフェラルへのレジスタアクセスをサポートする。PACはSVD2RUST でSVDから自動生成されたものがベースとなる。SVD(System View Description) はCMSIS-SVDで定められているインターフェイスで、ペリフェラルのレジスタをXMLベースで記述したもの。チップベンダから提供される。

MACとPACの上にHAL(Hardware Abstruction Layer)があり、チップの機能レベル(GPIOなど)のAPIを提供している。

さらにその上にBSPがボードレベルの機能(LEDやスイッチなど)を提供している。

このテンプレート・プロジェクトは BSP を使っているが`Cargo.toml` の設定を編集することで HALを使うようにすることもできる。

* BSP(rp-pico)
    + HAL(rp2040-hal)
        - PAC(rp2040-pac)
        - MAC(cortex-m)

このテンプレートでは、BSP が rp-pico。それが rp2040-halとrp2040-boot2を読み込む。rp2040-halはrp2040-pacを読み込む。
また、cortex-MのMACである`cortex-m`も別途読み込まれる。

### rp2040-boot2

RP2040というチップは、ユーザ・ファームウエア用の内蔵フラッシュが無い。フラッシュはQSPIで外付けされる。外付けされたフラッシュの先頭領域にかかれているブートローダが`rp2040-boot2`だ。ファームウエア本体を残りのQSPIから読み込んで実行する機能を持っている。

boot2本体はアセンブラで書かれているが数種類のバリエーションがある。

* BOOT_LOADER_W25Q080: rp-picoで使われている W25Q16JVは、このboot2 と互換性がある。
* BOOT_LOADER_RAM_MEMCPY: boot2の内容をSRAMにコピーして実行するもの。QSPI Flashを書き換えたい時などに使う(あとで使う)。
* 他にも、数種類の QSPI Flash チップをサポートしている。
* サイズは 0x100(256byte)。末尾4バイトはCRC32のチェックサムとなっている。

Raspberry Pi Picoボードには W25Q16JVが搭載されているので、rp-pico BSP の中で、それご互換性がある `BOOT_LOADER_W25Q080`が指定されている。
あとで、BSPを使わずに直接 rp2040-boot2を使うように変更する。

### cortex-m-rt

cortex-m-rt はCortex-M MCUの起動に関する部分をサービスするクレート。チップとは独立していて rust-embedded プロジェクトの成果物。

* いくつかのリンカスクリプト・フレームワークを提供し、コードを適切なアドレスに配置する。
    + リセット・ベクタ、割り込みベクタの割当も cortex-m-rt が担当する。
* スタートアップ・ルーチンを提供し、ユーザ・アプリが `#[entry]` と指定した関数を呼び出す。

#### リンカ・スクリプト

このテンプレートでは `.cargo/Config.toml`によってリンカオプションが指定されている。

```
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "probe-rs run --chip RP2040 --protocol swd"
```

`cargo run`したときのタスクランナー。`probe-rs` を使ってファームウエアを実行するように記述されている。

`rustflags`は`cargo`から`rustc`に渡される引数。そこからさらに`link-arg`はリンカに渡される引数。

```
rustflags = [
  "-C", "linker=flip-link",
  "-C", "link-arg=--nmagic",
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=-Tdefmt.x",
  "-C", "inline-threshold=5",
  "-C", "no-vectorize-loops",
]
```

`cortex-m-rt`が `link.x`を提供し、`link.x`がプロジェクトの`memory.x`を読み込むようになっている。
また、`link.x`は`rp2040-pac`が提供する`device.x`も読み込み、チップのメモリマップに合わせてリンクをアロケーションする。
また、`link.x`は、割り込みベクタのためのセクション(`.vector_table`)を提供する。

割り込みベクタを有効にするのは boot2 の役割。PPB の中にある VTOR(Vector Table Offset Register)に、割り込みベクタのアドレスをセットする。

また、`Cargo.toml` の方には、release build時にLTOを含む最適化を実施するように設定されている。


### panic-probe

`probe-rs`が提供するパニックハンドラ。パニック時にスタックトレースが出力されるのがデバッグに便利。
メッセージ出力インターフェイスとして`print-rtt`か`print-defmt`を選べる。テンプレート・プロジェクトでは`print-defmt`を使うようになっている。

### VS Code debugger

プロジェクト・テンプレートには `.vscode/launch.json` も付属している。`Cortex-Debug` など必要な拡張がインストールされていれば、VS CodeからGUIでデバッグが可能となる。今回はワークスペースのサブプロジェクトとして構成している。ビルドされたバイナリのパスが、ワークスペースの`target`を指すように修正しなければならない。

```launch.json
    "configurations": [
        {
            "coreConfigs": [
                {
                    "programBinary": "../target/thumbv6m-none-eabi/debug/rp2040-project-template",
```

また、launch.json中にコメントされているが、rp2040.svdを保存しておけば、デバッガの変数ビューで、ペリフェラル・レジスタが表示される。

# bootloader プロジェクトの作成

テンプレートプロジェクトをコピーして `bootloader`プロジェクトを作成する。

```
bootloader
├── src
│  └── main.rs
├── memory.x
├── Cargo.toml
├── build.rs
├── .vscode
│  ├── settings.json
│  ├── rp2040.svd
│  └── launch.json
├── .gitignore
└── .cargo
   └── config.toml
```

プロジェクト名を `rp2040-project-template`から`bootloader`に直したり、不要なファイルを消したりと、必要な修正を行う。

まずはこの段階で`cargo build`でビルドが通って、`cargo run`で正常動作することを確認。

