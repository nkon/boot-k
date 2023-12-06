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
- [自作するブートローダの機能](#自作するブートローダの機能)
- [メモリ・マップの設計](#メモリマップの設計)
  - [開発のステップ](#開発のステップ)
- [bootloader プロジェクトの作成](#bootloader-プロジェクトの作成)
  - [`rp-pico`というBSPへの依存をなくす](#rp-picoというbspへの依存をなくす)
  - [メモリマップを設計どおりに修正する](#メモリマップを設計どおりに修正する)
  - [UARTを使えるようにしておく](#uartを使えるようにしておく)
- [`bootloader`をもとに`app-blinky`を作る](#bootloaderをもとにapp-blinkyを作る)
  - [cargo-binutils](#cargo-binutils)
- [bootloaderから app-blinkyに制御を移す。](#bootloaderから-app-blinkyに制御を移す)
  - [boot2 が、自分自身のコードからアプリケーション(この場合は bootloader/main.rs#main())に制御を移す方法を調べる](#boot2-が自分自身のコードからアプリケーションこの場合は-bootloadermainrsmainに制御を移す方法を調べる)
    - [参考](#参考)
  - [`bootloader`が`app-blinky`を呼ぶ](#bootloaderがapp-blinkyを呼ぶ)
- [app-blinkyの署名を検証する](#app-blinkyの署名を検証する)
  - [ヘッダ構造体の定義とマップ](#ヘッダ構造体の定義とマップ)
    - [lib クレート、bin クレート](#lib-クレートbin-クレート)
  - [メモリからの読み込み](#メモリからの読み込み)
- [マルチ・ターゲット・ライブラリ](#マルチターゲットライブラリ)
    - [プロファイル](#プロファイル)
    - [イメージ操作ツール](#イメージ操作ツール)
    - [クロスアーキテクチャライブラリ](#クロスアーキテクチャライブラリ)
  - [イメージの署名を検証する](#イメージの署名を検証する)
  - [QSPI フラッシュメモリの操作](#qspi-フラッシュメモリの操作)


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


# 自作するブートローダの機能

* イメージの署名を検証して、正しい場合のみ起動する。
* 新しいイメージがあれば、古いイメージをアップデートして起動する。

# メモリ・マップの設計

* bootloader
    * .boot2: boot2が格納される。0x1000_0000から256B(=0x100)。末尾4BはCRC。
        * boot2は `rp2040-boot2`によってバイナリで供給される。
    * その後に `.vector_table`。192B(=0xc0)。
    * その跡に `.text`。0x1000_01c0から。
    * 合計 0x2_0000(128KB)

* application
    * .image_header: 256B(0x100)
    * その後に `.vector_table`。192B(0xc0)。
    * その跡に `.text`。
    * 合計 0xe_0000(896KB)
    * アップデート用にも同量のメモリが必要。

基本的な動作だと、Internal ROM => boot2 => bootloader本体へ実行を移す、という流れになる。
しかし、それだとbootloaderはXIPモードで動く。その場合、QSPI Flashは読み込み専用でマップされる。
なので、applicationの書き換えができない。

* Option #1: コピーコードをSRAM上で動くように書く。
* Option #2: boot2 を RAM モードにする。

| Address   | size           | Physical                      |project      |Segment      |  Address  | size          | Alias                   |
|-----------|----------------|-------------------------------|-------------|-------------|-----------|---------------|-------------------------|
|0x0000_0000|  16K(    0x400)| Internal ROM                  |             |             |0x0000_0000|               |ROM_BASE                 |
|           |                |                               |             |             |           |               |                         |
|0x1000_0000|2048K(0x20_0000)| QSPI Flash(XIP)               |bootloader   |.boot2       |0x1000_0000|0x100(256B)    |total 0x2_0000(128KB)    |
|           |                |                               |             |.vector_table|0x1000_0100|0x0c0(192B)    |                         |
|           |                |                               |             |.text        |0x1000_01c0|               |                         |
|           |                |                               |             |             |0x1002_0000|               |                         |
|           |                |                               |application  |.image_header|0x1002_0000|0x100(256B)    |total 0xe_0000(896KB)    |
|           |                |                               |             |.vector_table|0x1002_0100|0x0c0(192B)    |                         |
|           |                |                               |             |.text        |0x1002_01c0|               |                         |
|           |                |                               |             |             |0x1010_0000|               |                         |
|           |                |                               |app_update   |.image_header|0x1010_0000|0x100(256B)    |total 0xe_0000(896KB)    |
|           |                |                               |             |.vector_table|0x1010_0100|0x0c0(192B)    |                         |
|           |                |                               |             |.text        |0x1010_01c0|               |                         |
|           |                |                               |             |             |0x101e_0000|               |                         |
|           |                |                               |swap         |             |0x101e_0000|0x2_0000(128KB)|                         |
|           |                |                               |             |             |0x1020_0000|               |QSPI_END                 |
|           |                |                               |             |             |           |               |                         |
|0x2000_0000| 256K( 0x4_0000)| SRAM                          |             |0x2000_000000|0x2000_0000|0x1_0000(64KB) |SRAM_BASE                |
|           |                |                               |             |0x2001_000000|0x2001_0000|0x1_0000(64KB) |SRAM1_BASE               |
|           |                |                               |             |0x2002_000000|0x2002_0000|0x1_0000(64KB) |SRAM2_BASE               |
|           |                |                               |             |0x2003_000000|0x2003_0000|0x1_0000(64KB) |SRAM3_BASE               |
|           |                |                               |             |             |0x2004_0000|               |SRAM_STRIPED_END         |
|           |                |                               |             |0x2004_000000|0x2004_0000|  0x1000( 4KB) |SRAM4_BASE               |
|           |                |                               |             |0x2004_100000|0x2004_1000|  0x1000( 4KB) |SRAM5_BASE               |
|           |                |                               |             |             |0x2004_2000|               |SRAM_END                 |
|           |                |                               |             |             |           |               |                         |
|0x4000_0000|                | APB Peripherals               |             |             |0x4000_0000|               |                         |
|           |                |                               |             |             |           |               |                         |
|0x5000_0000|                | AHB-Lite Peripherals          |             |             |0x5000_0000|               |                         |
|           |                |                               |             |             |           |               |                         |
|0xd000_0000|                | IOPORT Registers              |             |             |0xd000_0000|               |                         |
|           |                |                               |             |             |           |               |                         |
|0xe000_0000|                | Cortex-M0+ internal registers |             |             |0xe000_0000|               |                         |
|           |                |                               |             |             |           |               |                         |

## 開発のステップ

1. rp2040_project_template をもとに bootloaderを作る。メモリマップは下記の設計にあわせる。
2. rp2040_project_template をもとに applicationとしてapp-blinkyを作る。
3. bootloaderから app-blinkyに制御を移す。
4. bootloaderがapp-blinkyに制御を移す前に .image_header の署名を検証する。
5. bootloaderをRAMにコピーして実行する。
6. bootloader は app_updateが存在したら、app_update => application にイメージをコピーして実行する。
7. イメージのコピーは swap を使って行い、失敗したら、古いイメージに戻して起動する。


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

ここから必要な修正を加えていく。

## `rp-pico`というBSPへの依存をなくす

* `Cargo.toml`から`[dependencies]`=>`rp-pico = "0.8"`を削除。
* `rp2040-hal = { version="0.9", features=["rt", "critical-section-impl"] }`と`rp2040-boot2 = "0.3"`を有効にする。
* `src/main.rs`で、BSPに依存している部分を、直接 `use`したり`rp3040-hal`への依存に変更する。

```main.rs
/// bsp を経由せずに直接 cortex_m_rt::entry を use する。エントリーポイントを指定するための `#[entry]`が使えるようになる。
-use bsp::entry;
+use cortex_m_rt::entry;

-use rp_pico as bsp;

-use bsp::hal::{
+use rp2040_hal::{
     clocks::{init_clocks_and_plls, Clock},
+    gpio::Pins,
     pac,
     sio::Sio,
     watchdog::Watchdog,
 };
 
/// bsp を経由せずに、直接 .boot2 セクションを指定する。
/// ここでは W25Q080 を指定。rpi-pico のW25Q16JV からXIP実行する
+#[link_section = ".boot2"]
+#[used]
+pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;


/// BSP を使わずに、HAL, GPIOを使ってLEDに繋がっているピンを指定する。
-    let pins = bsp::Pins::new(
+    let pins = Pins::new(

/// BSP を使わずに、HAL, GPIOを使ってLEDに繋がっているピンを指定する。ボード上ではGPIO25にLEDが繋がっている。
-    let mut led_pin = pins.led.into_push_pull_output();
+    let mut led_pin = pins.gpio25.into_push_pull_output();
```

## メモリマップを設計どおりに修正する

上の設計のとおり、bootloaderが使うメモリを `boot2`込みで 0x2_0000に変更する。もともとこの領域に収まっているので、大きな違いはない。

```
 MEMORY {
     BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
-    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
+    FLASH : ORIGIN = 0x10000100, LENGTH = 0x20000 - 0x100
     RAM   : ORIGIN = 0x20000000, LENGTH = 256K
 } 
```
## UARTを使えるようにしておく

なにかと便利だし RTT に依存しないようにするために UART を使えるようにしておく。

```main.rs
/// uart のHALをuseする。fugitは周波数や時刻の演算用ライブラリ
 use rp2040_hal::{
     clocks::{init_clocks_and_plls, Clock},
+    fugit::RateExtU32, // time calculation library
     gpio::Pins,
     pac,
     sio::Sio,
+    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
     watchdog::Watchdog,
 };
 
/// UARTの初期化
+    // Set up UART on GP0 and GP1 (Pico pins 1 and 2)
+    let uart_pins = (pins.gpio0.into_function(), pins.gpio1.into_function());
+    // Need to perform clock init before using UART or it will freeze.
+    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
+        .enable(
+            UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
+            clocks.peripheral_clock.freq(),
+        )
+        .unwrap();

/// メッセージの出力は `write_full_blocking()`で。引数はUTF-8ではなくバイト列で。
+    uart.write_full_blocking(b"bootloader stated...\r\n");

/// ビルド設定を出力するようにしておくと便利
+    #[cfg(debug_assertions)]
+    uart.write_full_blocking(b"bootloader debug build\r\n");
+
+    #[cfg(not(debug_assertions))]
+    uart.write_full_blocking(b"bootloader release build\r\n");

     loop {
-        info!("on!");
+        uart.write_full_blocking(b"bootloader on!\r\n");
         led_pin.set_high().unwrap();
         delay.delay_ms(500);
-        info!("off!");
+        uart.write_full_blocking(b"bootloader off!\r\n");
         led_pin.set_low().unwrap();
         delay.delay_ms(500);
     }
 }
```

UARTの出力は `cu`など、好みのターミナルソフトで表示できる。`cu`の場合、終了は`~.`。

```
❯ sudo cu -l /dev/tty.usbmodem13202 -s 115200
Connected.
bootloader stated...
bootloader debug build
bootloader on!
bootloader off!
bootloader on!
bootloader off!
bootloader on!
bootloader off!
bootloader on!
bootloader off!
bootloader on!
~.

Disconnected.
```


# `bootloader`をもとに`app-blinky`を作る

すでにLEDを点滅する機能は`bootloader`に存在するが、`bootloader`から起動されるアプリケーションとして、設計されたアドレスに配置されてイメージヘッダをもつ `app-blinky`を作成する。

* もともと `.boot2`があった位置に`.image_header`を配置する。
* 開始アドレスを修正する。
* メッセージなどの微修正。

```Cargo.toml
 [package]
-name = "bootloader"
+name = "app-blinky"
```

```memory.x
 MEMORY {
-    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
-    FLASH : ORIGIN = 0x10000100, LENGTH = 0x20000 - 0x100
+    IMAGE_HEADER : ORIGIN = 0x10020000, LENGTH = 0x100
+    FLASH : ORIGIN = 0x10020100, LENGTH = 0xe0000 - 0x100
     RAM   : ORIGIN = 0x20000000, LENGTH = 256K
 }
 
-EXTERN(BOOT2_FIRMWARE)
-
 SECTIONS {
-    .boot2 ORIGIN(BOOT2) :
+    .image_header ORIGIN(IMAGE_HEADER) :
     {
-        KEEP(*(.boot2));
-    } > BOOT2
-} INSERT BEFORE .text;
+        KEEP(*(.image_header));
+    } > IMAGE_HEADER
+} INSERT BEFORE .text;
```

```main.rs
// .boot2 セクションのかわりに .image_header セクションを配置する
-#[link_section = ".boot2"]
+#[link_section = ".image_header"]
 #[used]
-pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
+pub static IMAGE_HEADER: [u8; 256] = [0u8; 256];

/// あとはメッセージの修正など

-    uart.write_full_blocking(b"bootloader stated...\r\n");
+    uart.write_full_blocking(b"app-blinky stated...\r\n");
 
     #[cfg(debug_assertions)]
-    uart.write_full_blocking(b"bootloader debug build\r\n");
+    uart.write_full_blocking(b"app-blinky debug build\r\n");
 
     #[cfg(not(debug_assertions))]
-    uart.write_full_blocking(b"bootloader release build\r\n");
+    uart.write_full_blocking(b"app-blinky release build\r\n");
 
 
     loop {
-        uart.write_full_blocking(b"bootloader on!\r\n");
+        uart.write_full_blocking(b"app-blinky on!\r\n");
         led_pin.set_high().unwrap();
         delay.delay_ms(500);
-        uart.write_full_blocking(b"bootloader off!\r\n");
+        uart.write_full_blocking(b"app-blinky off!\r\n");
         led_pin.set_low().unwrap();
         delay.delay_ms(500);
     }
```

まだ、今の段階では `cargo run`しても`bootloader`しか動作しない。

`cargo objdump`して、セクションが設計通りのアドレスに配置されているかどうかを確認しておく。

## cargo-binutils

rust-embedded プロジェクトが出している [cargo-binutils](https://github.com/rust-embedded/cargo-binutils) を入れて置けば、ほぼ gnu binutils 互換で、バイナリの情報を調べることができる。`--`より前のオプションは`cargo`に対するもの、`--`より後ろのオプションは`objdump`に対するもの。ターゲットのバイナリは cargo の情報から適切に選択される。

```
❯ cargo objdump -v -- --headers
"~/.rustup/toolchains/stable-aarch64-apple-darwin/bin/cargo" "build" "--message-format=json"
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /.../boot-k/bootloader/Cargo.toml
workspace: /.../boot-k/Cargo.toml
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /.../boot-k/app-blinky/Cargo.toml
workspace: /.../boot-k/Cargo.toml
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /.../boot-k/rp2040-project-template/Cargo.toml
workspace: /.../boot-k/Cargo.toml
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
cd "/.../boot-k/target/thumbv6m-none-eabi/debug" && "rust-objdump" "--triple" "thumbv6m-none-eabi" "app-blinky" "--headers"

app-blinky:     file format elf32-littlearm

Sections:
Idx Name            Size     VMA      LMA      Type
  0                 00000000 00000000 00000000 
  1 .vector_table   000000c0 10020100 10020100 DATA
  2 .image_header   00000100 10020000 10020000 DATA
  3 .text           0000bda0 100201c0 100201c0 TEXT
  4 .rodata         00001bf4 1002bf60 1002bf60 DATA
  5 .data           00000038 2003fbb8 1002db54 DATA
  6 .gnu.sgstubs    00000000 1002dba0 1002dba0 TEXT
  7 .bss            0000000c 2003fbf0 2003fbf0 BSS
  8 .uninit         00000400 2003fbfc 2003fbfc BSS
  9 .defmt          00000006 00000000 00000000 
 10 .debug_abbrev   00006764 00000000 00000000 DEBUG
 11 .debug_info     000cd976 00000000 00000000 DEBUG
 12 .debug_aranges  000079a8 00000000 00000000 DEBUG
 13 .debug_ranges   00024248 00000000 00000000 DEBUG
 14 .debug_str      00103db5 00000000 00000000 DEBUG
 15 .debug_pubnames 00048b9e 00000000 00000000 DEBUG
 16 .debug_pubtypes 0004df35 00000000 00000000 DEBUG
 17 .comment        00000040 00000000 00000000 
 18 .ARM.attributes 00000032 00000000 00000000 
 19 .debug_frame    00016bbc 00000000 00000000 DEBUG
 20 .debug_line     0005b76d 00000000 00000000 DEBUG
 21 .debug_loc      000013bc 00000000 00000000 DEBUG
 22 .symtab         00005b20 00000000 00000000 
 23 .shstrtab       0000010b 00000000 00000000 
 24 .strtab         0000e3e6 00000000 00000000 
```


# bootloaderから app-blinkyに制御を移す。

## boot2 が、自分自身のコードからアプリケーション(この場合は bootloader/main.rs#main())に制御を移す方法を調べる

```
❯ cargo objdump -v -- --disassemble-all > asm.S
```

のようにディスアセンブルすれば、命令とアドレスの対応が手に入る。

```
bootloader:	file format elf32-littlearm

Disassembly of section .vector_table:

10000100 <__vector_table>:                   # bootloader のコードの先頭
10000100: fbb8 2003    	<unknown>            # 0x1000_0100 には SP初期値(0x2003_fbb8)が入っている

10000104 <__RESET_VECTOR>:
10000104: 01c1         	lsls	r1, r0, #0x7   # 0x1000_0104 には コードの先頭アドレス(0x1000_01c1)が入っている
10000106: 1000         	asrs	r0, r0, #0x20  # ディスアセは「当てはめ」

10000108 <__reset_vector>:
10000108: ab0d         	add	r3, sp, #0x34    # そこから先も割り込みハンドラのアドレス。
1000010a: 1000         	asrs	r0, r0, #0x20
1000010c: bf45         	<unknown>
1000010e: 1000         	asrs	r0, r0, #0x20
		...
Disassembly of section .boot2:

10000000: b500         	push	{lr}           # boot2のコードの先頭
10000002: 4b32         	ldr	r3, [pc, #0xc8]         @ 0x100000cc
		...
1000009c: 4812         	ldr	r0, [pc, #0x48]         @ 0x100000e8   # 最終的にここにやってくる
1000009e: 4913         	ldr	r1, [pc, #0x4c]         @ 0x100000ec
100000a0: 6008         	str	r0, [r1]
100000a2: c803         	ldm	r0, {r0, r1}
100000a4: f380 8808    	msr	msp, r0
100000a8: 4708         	bx	r1               # ここで boot2 の終了、bootloaderの実行開始
		...
100000e8: 0100         	lsls	r0, r0, #0x4   # 0x1000_00e8 には 0x1000_0100が格納(bootloaderの先頭アドレス)
100000ea: 1000         	asrs	r0, r0, #0x20  # ディスアセは「当てはめ」
100000ec: ed08 e000    	<unknown>            # 0x1000_00ec には 0xe0000_ed08 が格納(PPB_BASE+VTOR_OFFSET)
```

しかし、それでは読みにくいので https://github.com/rp-rs/rp2040-boot2/blob/main/src/boot2_w25q080.S からソースコードを読んでいく。

コードの動作はアセンブラのほうがわかりやすいが、アドレスの中身はディスアセンブラのほうがわかりやすい。

ペリフェラル(SSI)の初期化と、QSPI Flash の設定を行ったあと、最終的に実行されるコードのアセンブラバージョンは次。


https://github.com/rp-rs/rp2040-boot2/blob/main/src/include/boot2_helpers/exit_from_boot2.S#L20

```asm
vector_into_flash:
    ldr r0, =(XIP_BASE + 0x100)               // #define XIP_BASE 0x10000100
    ldr r1, =(PPB_BASE + M0PLUS_VTOR_OFFSET)  // #define PPB_BASE 0xe0000000
                                              // #define M0PLUS_VTOR_OFFSET 0x0000ed08
    str r0, [r1]            // VTOR <= 0x1000_0100
    ldmia r0, {r0, r1}      // r0 <= [r0], r1 <= [r1]
    msr msp, r0             // msp <= r0     msp=stack pointer
    bx r1                   // 最終的に bx r1 でr1のアドレスにジャンプする 
```

**とても興味深いコードだ**。

* まず、`r0` に `XIP_BASE+0x100(=0x1000_0100)` という値をロード。実際のマシン語では、32bit即値は(固定長命令の中にはエンコードできないので)別アドレス(`0x1000_00e8`)に格納されているのを、PC間接参照でロードする(`ldr	r3, [pc, #0xc8]`)。
* 次に、`r1` に `PPB_BASE+M0PLUS_VTOR_OFFSET(=0xe000_ed08)` という値をロード。`VTOR` はVector Table Offset Registerのことで、その名のとおり、割り込みベクタ・テーブルの先頭アドレスを示す。
* そして、`r0`の値を`r1`が指すアドレスにストアする(インテル形式とは逆向き)。つまり、`VTOR` が`0x1000_0100`を指す。ここには、リンカによって、`bootloader`の`.vector_table`が居る。
* `ldmir`はレジスタ復元命令。`POP`のようなもの。`r0`が指し示すアドレスから始まるメモリの内容を、`{r0, r1}`の2つのレジスタに格納する。つまり、`r0`には`r0`が指す`.vector_table[0]`の内容(=SP初期値=`0x2003_fbb8`)が、`r1`には`.vector_table[1]`の内容(=コードの先頭アドレス=`0x1000_01c1`)が格納される。
* `msr`はスタックポインタを更新する専用命令。`r0`の内容(=SP初期値=`0x2003_ffb8`)が`msp`にセットされる。
* `bx r1`で`r1`の指すアドレス(=`0x1000_01c1`)にジャンプする。ジャンプの場合、アドレス末尾のビットが`1`だと、それを`0`に変更して、little endian モードで実行する。

### 参考

* `VTOR`の解説 https://developer.arm.com/documentation/dui0662/b/Cortex-M0--Peripherals/System-Control-Block/Vector-Table-Offset-Register
* `ldmir`命令の解説は https://www.mztn.org/slasm/arm05.html
* `msr`命令の解説は http://idken.net/posts/2017-12-11-arm_asm2/

## `bootloader`が`app-blinky`を呼ぶ


これと同様に bootloader/main.rsを実装すればよい。

* インラインアセンブラを使う。

```rust 
+use core::arch::asm;
```

* 次の部分が制御を移す本質。
    + r0 に、移動先のPC(プログラムカウンタ)の値をセット
    + r1 に、新しいスタックポインタの値をセット
    + 今回はアドレステーブルを元に即値で書いたが、移植性をよくするなら各種定数から計算するほうが良い。
* インラインアセンブラ中では、`fmt!`的に`{}`は変数と解釈されるので、`{{ }}`とエスケープする。

```rust
+    unsafe {
+        asm!(
+            "ldr r0, =0x10020100",
+            "ldr r1, =0xe000ed08",
+            "str r0, [r1]",
+            "ldmia r0, {{r0, r1}}",
+            "msr msp, r0",
+            "bx r1",
+        );
+    };
```

`cargo run`すると、シリアルコンソールに、`bootloader`のメッセージと`app-blinky`のメッセージが表示される。

これで、`bootloader`が`app-blinky`を起動することができた!!!

```
❯ sudo cu -l /dev/tty.usbmodem13202 -s 115200
Connected.
bootloader stated...
bootloader debug build
app-blinky stated...
app-blinky debug build
app-blinky on!
app-blinky off!
app-blinky on!
app-blinky off!
~.

Disconnected.
```

# app-blinkyのイメージヘッダ

ブートローダがブートするアプリケーションは、単なる実行イメージだけでなく、付加的な情報をイメージヘッダとして保持させる。

## ヘッダ構造体の定義とマップ

今は256byteのゼロ埋めされているヘッダ領域だが、中身の構造を作っていく。

### lib クレート、bin クレート

今の構造は次のようになっている。

ちなみに、[`tre`](https://github.com/dduan/tre) コマンドは`tree`コマンドの改良版みたいなもので、色々便利になっている。

```

~/s/r/boot-k on  main [!?] via 🦀 v1.73.0 
❯ tre 
[0] .
├── [1] app-blinky
│   └── [13] src
│       └── [14] main.rs
├── [15] bootloader
│   ├── [16] src
│   │   └── [19] main.rs
├── [31] Cargo.lock
├── [32] Cargo.toml
└── [33] NOTE.md
```

`app-blinky/`の下に`src/main.rs`があり、ここから`app-blinky`という実行ファイルが作られる。また`bootloader`の下に`src/main.rs`があり、ここから`bootloader`という実行ファイルが作られる。

rustのプラクティスとして、「実行ファイルを作る場合でも、ほとんどの機能をライブラリとして実装する」というものがある。`main.rs`からは実行ファイルが作られ、`lib.rs`からはライブラリが作られる。実行ファイルは実行形態なので結合テストができないがライブラリは結合テストが実施される。そのために、`bootloader/src`の下に、`lib.rs`とライブラリの実装(この場合は`image_header.rs`)を作る、

[TRPL 12.3](https://doc.rust-jp.rs/book-ja/ch12-03-improving-error-handling-and-modularity.html)参照

```
 tre 
[0] .
├── [1] app-blinky
│   └── [13] src
│       └── [14] main.rs
├── [15] bootloader
│   ├── [16] src
│   │   ├── [17] image_header.rs
│   │   ├── [18] lib.rs
│   │   └── [19] main.rs
├── [31] Cargo.lock
├── [32] Cargo.toml
└── [33] NOTE.md
```

このようにすると、`bootloader`というライブラリが作られ、`bootloader/src/main.rs`はそれを use する。

`bootloader/src/lib.rs`では、`bootloader::image_header`というライブラリをエクスポートする。

```bootloader/src/lib.rs
#![no_std]
pub mod image_header;
```
`bootloader/src/image_header.rs`では、`bootloader::image_header`というライブラリを実装する。中身は構造体の定義とそれを扱う関数。

```bootloader/src/image_header.rs
use core::ptr;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ImageHeader {
    pub header_magic: u32,  // 4
    pub header_length: u16, // +2 = 6
    pub hv_major: u8,       // +1 = 7
    pub hv_minor: u8,       // +1 = 8

    pub iv_major: u8,     // +1 = 9
    pub iv_minor: u8,     // +1 = 10
    pub iv_revision: u16, // +2 = 12
    pub iv_build: u32,    // +4 = 16

    pub image_length: u32,    // +4 = 20
    pub signature: [u8; 128], // +128 = 148

    pub padding: [u8; 104], // +104 = 252
    pub crc32: u32,         // +4 = 256
}

pub fn load_from_addr(addr: u32) -> ImageHeader {
    unsafe { ptr::read_volatile(addr as *const ImageHeader) }
}
```

`bootloader/src/main.rs`では`bootloader`ライブラリから`image_header`モジュールを`use`する。ここでは`app_blinky`のイメージヘッダ領域(0x1002_0000)を読み込んで、一部を表示している。


```bootloader/src/main.rs
use bootloader::image_header;

...

    let ih = image_header::load_from_addr(0x1002_0000);
    info!(
        "{:x} {:x} {:x} {:x}",
        ih.header_magic, ih.header_length, ih.hv_major, ih.hv_minor
    );
```

一方、`app-blinky`の側では、`Cargo.toml`で相対パスを用いて、ローカル・ライブラリの使用を宣言する。

```app-blinky/Cargo.toml
[dependencies.bootloader]
path = "../bootloader"
```

`app-blinky/src/main.rs`で次のようにライブラリを使うことができる。

`header_magic`は中二病っぽく、リートコードを使って"bootload"っぽくしてみた。

```app-blinky/src/main.rs
#[link_section = ".image_header"]
#[used]
pub static IMAGE_HEADER: image_header::ImageHeader = image_header::ImageHeader {
    header_magic: 0xb00410ad,
    header_length: 256,
    hv_major: 0,
    hv_minor: 1,
    iv_major: 0,
    iv_minor: 1,
    iv_revision: 0,
    iv_build: 1234,
    image_length: 0xe_0000,
    signature: [0u8; 128],
    padding: [0u8; 104],
    crc32: 0,
};
```

また、`app-blinky`側で`cargo run`しても、イメージの書き込み→リセットして実行しても、`bootloader`が実行されて、`app-blinky`が実行されない。書き込むだけのシェルスクリプト(`write_image.sh`)を作成しておく。

本来は`probe-rs`が提供する`cargo flash`がその役目を果たすはずだが、なぜかうまく動かない。しかも、失敗した以降、デバッガと全てのSWD通信がハングアップする。


```
#!/bin/bash

set -uex

arch=${arch:-"thumbv6m-none-eabi"}
debug=${debug:-"debug"}

probe-rs download --chip RP2040 --protocol swd ../target/${arch}/${debug}/app-blinky
probe-rs reset --chip RP2040 --protocol swd
```

## メモリからの読み込み

すでに、上でコード例をあげたが、任意のアドレスから読み込むには`core::ptr::read_volatile`が使える。アドレスの即値は `as *const T`に強制キャストする。`ImageHeader`は`#[repl(C)]`として宣言してあるので、C的なメモリ配置となり、`read_volatile`したメモリイメージを、そのままキャストすれば構造体にマップされる。

```bootloader/src/image_header.rs
use core::ptr;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ImageHeader {
...
}

pub fn load_from_addr(addr: u32) -> ImageHeader {
    unsafe { ptr::read_volatile(addr as *const ImageHeader) }
}
```

```
❯ cd app-blinky 

❯ cargo build                   # app-blinkyをビルドする
package:   /Users/nkon/src/rust/boot-k/rp2040-project-template/Cargo.toml
workspace: /Users/nkon/src/rust/boot-k/Cargo.toml
   Compiling app-blinky v0.1.0 (/Users/nkon/src/rust/boot-k/app-blinky)
    Finished dev [unoptimized + debuginfo] target(s) in 0.15s

❯ ./write_image.sh              # app-blinkyのイメージを書き込む
+ probe-rs download --chip RP2040 --protocol swd ../target/thumbv6m-none-eabi/debug/app-blinky
     Erasing sectors ✔ [00:00:00] [] 56.00 KiB/56.00 KiB @ 65.91 KiB/s (eta 0s )
 Programming pages   ✔ [00:00:01] [] 56.00 KiB/56.00 KiB @ 30.11 KiB/s (eta 0s )    Finished in 2.744s
+ probe-rs reset --chip RP2040 --protocol swd

❯ cd ../bootloader   

❯ cargo run                     # bootloader をビルド＆実行する
package:   /Users/nkon/src/rust/boot-k/rp2040-project-template/Cargo.toml
workspace: /Users/nkon/src/rust/boot-k/Cargo.toml
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `probe-rs run --chip RP2040 --protocol swd /Users/nkon/src/rust/boot-k/target/thumbv6m-none-eabi/debug/bootloader`
     Erasing sectors ✔ [00:00:00] [] 64.00 KiB/64.00 KiB @ 64.89 KiB/s (eta 0s )
 Programming pages   ✔ [00:00:02] [] 64.00 KiB/64.00 KiB @ 30.22 KiB/s (eta 0s )    Finished in 3.137s
INFO  Program start
└─ bootloader::__cortex_m_rt_main @ src/main.rs:31  
INFO  b00410ad 100 0 1          # magicなどの値が正常に読めている
└─ bootloader::__cortex_m_rt_main @ src/main.rs:86  
0
```

# マルチ・ターゲット・ライブラリ

いろいろ複雑になってきたので、ワーク・スペースを整理する。次のような構成にしたい。

とくに`blxlib`というライブラリは、`bootloader`や`app-blinky`のようなターゲット上で動作するバイナリからも、`bintool`のようなネイティブで動作するバイナリからも利用可能な、どちらのアーキテクチャに向けてもビルドすることができる、ライブラリだ。

たとえば、イメージヘッダのような情報は、ターゲット上で動くバイナリも必要としているし、ネイティブ環境で動くバイナリ操作ツールも必要としている。同一ソースであることで移植バグが防げる。

また、ターゲット環境向けのライブラリであっても、ネイティブ向けにもビルドすることができれば、論理的なテストは `cargo test` でネイティブ環境で実行することができる。

このような、マルチ・ターゲット・ライブラリは、Rustのクロス・コンパイル能力を活用している。

* bootloader : bootloaderプロジェクト。thumbv6m の実行ファイルを生成する
    + bootloader/src/main.rs : bootloaderバイナリの起点となるファイル
        - `use bootloader;`
        - `use blxlib;`
    + bootloader/src/lib.rs : bootloaderライブラリの起点となるファイル
* app-blinky: app-blinyプロジェクト。bootloaderから起動されるアプリケーション。thumbv6mの実行ファイルを生成する。
    + app-blinky/src/main.rs : app-blinkyバイナリの起点となるファイル。
        - `use bootloader;`
        - `use blxlib;`
* bintool: bintoolプロジェクト。バイナリ操作ツールのネイティブの実行ファイルを生成する。
    + bintool/src/main.rs: bintoolバイナリの起点となるファイル。
        - `use blxlib;`
* blxlib : blxlibプロジェクト。クロスプラットフォームライブラリ。ソースコードは共通でthumbv6m(bootloader, app-blinky)にも、ネイティブツール(bintool)にもビルドできる。
    + blxlib/src/lib.rs: blxlib : ライブラリの起点となるファイル。
    + blxlib/src/image_header.rs : イメージヘッダの構造を定める。bootloader, app-blinky, bintoolで使われる。
* tools : python などのスクリプト・ツール(予定)
* rp2040-project-template : 参照用のプロジェクトテンプレート。ワークスペース外。

```
├── bootloader                # bootloader プロジェクト thumbv6m, bin
│  ├── src
│  │  ├── main.rs           # bootloader(bin)
│  │  └── lib.rs            # bootloader(lib)
├── app-blinky                # app-blinky プロジェクト thumbv6m, bin
│  ├── src
│  │  └── main.rs
├── blxlib                    # blxlib プロジェクト thumbv6m, lib
│  ├── src
│  │  ├── lib.rs            # blxlib(lib)
│  │  └── image_header.rs
├── bintool                   # bintool プロジェクト native, bin
│  ├── src
│  │  └── main.rs
├── tools                     # スクリプト類
│  ├── requirments.txt
│  └── bintool.py
├── target                    # workspace のビルドディレクトリ
│  ├── thumbv6m-none-eabi    # thumbv6m 版のビルドディレクトリ
│  │  ├── debug
│  │  │  ├── libbootloader.rlib  # bootloader(lib) thumbv6m, lib
│  │  │  ├── bootloader    # bootloader(bin) thumbv6m, bin
│  │  │  ├── app-blinky    # app-blinky thumbv6m, bin
│  ├── debug                 # ネイティブ版のビルドディレクトリ
│  │  ├── libblxlib.rlib    # blxlib(lib) ネイティブ
│  │  ├── bintool           # bintool(bin) ネイティブ
├── rp2040-project-template   # テンプレート・プロジェクト(workspace外)
│  ├── target                # ワークスペース外なので、サブディレクトリでビルド
│  │  ├── thumbv6m-none-eabi
│  │  │  ├── debug
│  │  │  │  ├── rp2040-project-template # rp2040-project-template thumbv6m, bin
│  ├── src
│  │  └── main.rs
```

### プロファイル

`rp2040-project-template`からコピーしたばかりの`bootloader/`や`app-blinky`などのサブプロジェクトでビルドするとつぎのようにwarningが出る。

```
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /.../boot-k/bootloader/Cargo.toml
workspace: /.../boot-k/Cargo.toml
```

メッセージに書いてあるように、`bootloader/Cargo.toml`に書かれている`[profile]`関係のセクションを、ワークスペース・ルートの`Cargo.toml`に移動する。`app-blinky/Cargo.toml`についても同様。

### イメージ操作ツール

今の段階ではスケルトンだが、将来的に`bintool`というイメージ作成ツールを構想している。
`app-blinky`の実行ファイルに署名したり、アップデート用のイメージを生成したりするツールとしたい。`bintool`はネイティブ(今の場合は aarch64-darwin)で動作するツール。

```
❯ cargo new bintool --bin
❯ cd bintool
❯ cargo run  
Hello, world!
```


### クロスアーキテクチャライブラリ

イメージヘッダの情報は、bootloaderも知る必要があるし、app-blinkyも知る必要がある。また、bintoolにも共有したい。

ターゲット向けのライブラリは、`bootloader/src/lib.rs`に集約すれば良いが、クロス部分は別のクレート(blxlib)を作る。

これは、ツールなどのネイティブ環境だけでなく、`bootloader`のような組み込み環境でも使われるので、ライブラリ全体が`#![no_std]`である必要がある。

```
❯ cargo new blxlib --lib
## メッセージで出ているように、workspaceの`Cargo.toml`の`workspace.members`に`blxlib`を追加する。
❯ cd blxlib 
❯ cargo test
running 1 test
test tests::it_works ... ok
```

生成後のデフォルトで、ネイティブでのテストがパスする状態になっている。

これを、次のようにすれば bootloaderからも使える。

```bootloader/Cargo.toml
[dependencies.blxlib]
path = "../blxlib"
```

```bootloader/src/main.rs
use blxlib::image_header;
```

# イメージの署名を検証する

## probe-rsでバイナリを書き込む

イメージを直接操作するので、コンパイラのアウトプット(ELF)を`objcopy`でバイナリ形式にする=>バイナリを編集する=>probe-rsでバイナリを書き込む、というように変更する。

cargo-binutilsのobjcopyは、なぜかうまく動かなかったので、バラのbinutilsをインストールする。

```
❯ brew install arm-none-eabi-binutils
```

`arm-none-eabi-objcopy`で `-O binary`を指定すれば、バイナリフォーマットを得ることができる。これは、メモリイメージをベタ書きしたもの。

```
arm-none-eabi-objcopy -O binary ../target/${arch}/${debug}/app-blinky ../target/${arch}/${debug}/app-blinky.bin
```

<<<ここで、イメージを編集する:後述>>>

probe-rs はELFだけでなくバイナリも書き込めるが、ドキュメントはなく、ソースを掘る必要がある。`--format bin`でバイナリフォーマットであることを明示し、`--base-address`オプションで開始アドレスを指定(バイナリフォーマットは開始アドレスの情報が含まれていない)、`--skip`オプションで、イメージ先頭のスキップする長さを指定する。

```
probe-rs download --chip RP2040 --protocol swd --format bin --base-address 0x10020000 --skip -0 ../target/${arch}/${debug}/app-blinky.base
```

## RSAとCRCのライブラリ

署名機能のために組み込み可能なRSAライブラリのRust実装を探してみた。良さそうなのは次の2つ。

* [RustCrypt](https://github.com/RustCrypto/RSA)。Pure RustのRSAライブラリ。`no_std`対応は[パッチ](https://github.com/RustCrypto/RSA/pull/22)があるが、現在の対応についてはドキュメントが見つけられなかった。
* もう一つは[mbedtlsのRustラッパー](https://github.com/fortanix/rust-mbedtls)。ビルドが複雑になってしまう。

とりあえず、当初計画していたRSAの署名はいったん置いておいて、CRC32のチェックだけを実装することにする。

CRC32のライブラリは色々探してみたが、次が良さそう。

* [const-crc32](https://git.shipyard.rs/jstrong/const-crc32)。`const fn`で固定文字列のCRC32を計算するクレートだが、クレート計算部分がシンプルなので、可変文字列対応に移植するのも容易。テストケースを書いて、[CRC計算機](https://crccalc.com/?crc=0x00,0x00,0x00,0x00&method=crc32&datatype=hex&outtype=0)のCRC32(いくつか種類があるが無印のもの)との結果(Result列)を比較しておく。

```
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        let input = [0u8, 0u8, 0u8, 0u8];
        let result = crc32(&input);
        // https://crccalc.com/?crc=0x00,0x00,0x00,0x00&method=crc32&datatype=hex&outtype=0
        assert_eq!(result, 0x2144DF1C);

        // https://crccalc.com/?crc=hoge&method=crc32&datatype=ascii&outtype=0
        let input = "hoge".as_bytes();
        let result = crc32(&input);
        assert_eq!(result, 0x8B39E45A);
    }
}
```
テストは普通にホスト・ネイティブ環境で実行できる。

```
❯ cargo test 
   Compiling blxlib v0.1.0 (/.../boot-k/blxlib)
    Finished test [optimized + debuginfo] target(s) in 0.32s
     Running unittests src/lib.rs (/.../boot-k/target/debug/deps/blxlib-7519cf517afeeddd)

running 3 tests
test crc32::tests::test_crc32 ... ok
test image_header::tests::test_is_correct_magic ... ok
test image_header::tests::test_set_crc32 ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests blxlib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


