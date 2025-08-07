# raw91-tun

`raw91-tun` 是一个点对点的 TUN 隧道工具，它通过 IPv4 原始套接字 (raw socket) 工作，并使用 base91 编码来混淆载荷。该工具旨在为两个具有公网 IP 地址的主机之间创建一个简单的、未加密的隧道。

## 功能特性

  * **创建 TUN 设备:** 创建一个 TUN 网络接口用于路由 IP 流量。
  * **Raw Socket 通信:** 使用 IPv4 原始套接字在对等方之间发送和接收 TUN 流量。
  * **Base91 载荷编码:** 所有 TUN 流量都使用 base91 进行编码，以避免被深度包检测或基于协议的过滤。
  * **可配置:** 所有选项都可以通过 TOML 配置文件设置，并可通过命令行参数覆盖。

## 配置

您可以创建一个 `config.toml` 文件来配置 `raw91-tun`。所有选项也可以作为命令行参数传递，以覆盖文件中的设置。

这是一个 `config.toml` 的示例：

```toml
# 为 TUN 接口命名 (可选). 如果省略, 内核会选择一个 (例如, tun0)
# tun_name = "tun91"

# 内部 TUN 的 MTU (为 base91 编码和外部 IPv4 头部预留空间)
mtu = 1200

# 在 TUN 接口上分配 IPv4 地址 (可选; 也可以在 post_up 中完成)
tun_v4_addr = "10.91.0.2"
tun_v4_peer = "10.91.0.1"
tun_v4_mask = "255.255.255.0"

# (可选) 通过 post_up 命令配置 IPv6
#post_up = [
#  "ip -6 addr add fd91::2/64 dev tun91",
#  "ip -6 route add fd91::/64 dev tun91",
#]
#post_down = [
#  "ip -6 addr del fd91::2/64 dev tun91 || true",
#]

# 远端 raw IPv4 对等点 (另一个端点的公共/可路由地址)
raw_remote_v4 = "123.45.67.89"

# 绑定本地地址 (可选)
# raw_bind_v4 = "127.0.0.1"

# 自定义 IP 协议号 (0-255). 推荐使用实验性值 (如 253/254) 或您可控的数值。
ip_protocol = 41

# 外部 IPv4 路径 MTU 及丢包策略
outer_mtu = 1500
drop_if_exceeds = true

# 日志级别
log_level = "info"
```

### 配置选项

  * `tun_name`: TUN 接口的名称 (例如, "tun0")。
  * `mtu`: TUN 接口的最大传输单元。
  * `tun_v4_addr`: 分配给 TUN 接口的 IPv4 地址。
  * `tun_v4_peer`: P2P 隧道的目的 (对等) IPv4 地址。
  * `tun_v4_mask`: TUN 接口 IPv4 地址的子网掩码。
  * `post_up`: TUN 设备创建后要执行的 shell 命令列表。
  * `post_down`: 程序终止后要执行的 shell 命令列表。
  * `raw_remote_v4`: 远端对等点的公网 IPv4 地址。
  * `raw_bind_v4`: (可选) 用于绑定 raw socket 的本地 IPv4 地址。
  * `ip_protocol`: 用于 raw socket 通信的 IP 协议号 (0-255)。双方必须使用相同的协议号。
  * `outer_mtu`: raw socket 连接的估算路径 MTU。如果 `drop_if_exceeds` 为 true，大于此值的数据包将被丢弃。
  * `drop_if_exceeds`: 如果为 true，编码后大于 `outer_mtu` 的数据包将被丢弃。
  * `log_level`: 日志记录级别。可以是 `error`, `warn`, `info`, `debug`, 或 `trace` 之一。

## 依赖项

该项目的主要依赖项如下：

  * `anyhow`
  * `base91`
  * `clap`
  * `ctrlc`
  * `env_logger`
  * `log`
  * `nix`
  * `serde`
  * `socket2`
  * `toml`
  * `tun`

## 安装与使用

1.  **安装 Rust:** 如果您尚未安装 Rust，可以从 [rust-lang.org](https://www.rust-lang.org/) 获取。
2.  **构建项目:**
    ```bash
    cargo build --release
    ```
3.  **运行:**
    ```bash
    ./target/release/raw91-tun -c /path/to/your/config.toml
    ```
    您需要使用 `sudo` 或具有 `CAP_NET_ADMIN` 权限的用户来运行此程序。

## 许可证

该项目根据 Apache License, Version 2.0 进行许可。您可以在 `LICENSE` 文件中找到完整的许可证文本。