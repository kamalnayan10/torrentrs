# 🚀 TorrentRS

**TorrentRS** is a lightweight BitTorrent client built in **Rust**, created to dive deep into low-level network programming and understand the inner workings of the BitTorrent protocol. It showcases how to implement a basic peer-to-peer (P2P) file-sharing system — from connecting with peers to downloading pieces of a file concurrently and safely.

---

## ✨ Features

- **🧩 Peer Communication**
  - Implements BitTorrent handshake and peer messaging.
  - Supports peer discovery and dynamic connection handling.

- **📦 Piece Downloading**
  - Requests and downloads file pieces from multiple peers.
  - Ensures data integrity and reassembles the original file.

- **🛡️ Robust Error Handling**
  - Gracefully handles network interruptions and malformed peer messages.
  - Retry mechanisms and fail-safes for reliable downloads.

- **⚙️ Powered by Rust**
  - Leverages Rust's concurrency, safety, and performance.
  - Thread-safe architecture with minimal overhead.

---

## 🛠️ Getting Started

### 1. Clone the Repository
```bash
git clone https://github.com/kamalnayan10/torrentrs
cd torrentrs
```

### 2. Build the Project
```bash
cargo build --release
```

### 3. Run TorrentRS
```bash
cargo run --release -- path/to/file.torrent
```

Ensure the `.torrent` file is valid and contains peers or a tracker.

---

## 📚 Resources

- 📖 [BitTorrent Protocol Specification (BEP-3)](https://www.bittorrent.org/beps/bep_0003.html)

---

## 🧠 Why This Project?

TorrentRS is built as a learning tool to:

- Understand TCP socket programming and asynchronous I/O.
- Gain hands-on experience with binary protocols.
- Explore concurrency primitives in Rust.
- Deepen knowledge of P2P systems and real-world networking protocols.
