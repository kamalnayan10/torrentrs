# TorrentRS

### TorrentRS is a BitTorrent client built in Rust, following the Codecrafters tutorial to explore low-level network programming and the BitTorrent protocol. This project demonstrates the process of creating a basic peer-to-peer (P2P) file-sharing client, from establishing peer connections to managing file downloads.

### Features

  - Peer Communication: Handshakes, peer discovery, and message exchanges for initiating downloads.
  - Piece Downloading: Requests and receives pieces of the file from multiple peers.
  - Error Handling: Robust error handling for reliable data transfers.
  - Rust Performance: Utilizes Rust’s concurrency and memory safety features for optimal performance.

### Usage


Clone the Repository:
```
git clone https://github.com/your-username/torrentrs.git
cd torrentrs
```

Build the project:
```
cargo build --release
```

Start TorrentRS with a .torrent file:
```
cargo run path/to/file.torrent
```

### Resources

  - Tutorial: This project was guided by Codecrafters' [Build Your Own BitTorrent Client tutorial](https://app.codecrafters.io/courses/bittorrent/overview)
  - [BitTorrent Protocol: Official Protocol Documentation](https://www.bittorrent.org/beps/bep_0003.html)
