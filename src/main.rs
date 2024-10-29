use serde_json;
use serde_json::Value;
use std::{env, fs};
use sha1::{Sha1 , Digest};
use std::net::TcpStream;
use std::io::{Write, Read};
use url::form_urlencoded;
use tokio::net::TcpStream as tokioTcp;
use tokio::io::{AsyncWriteExt , AsyncReadExt};

// Available if you need it!
// use serde_bencode;

#[derive(Default)]
struct TorrentInfo{
    url:String,
    length:i64,
    hash:Vec<u8>,
    piece_length: i64,
    pieces:Vec<Vec<u8>>,
    port:u16,
    peer_id:String,
    peers_list: Vec<String>
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &[u8]) -> (Value,usize){
    if encoded_value[0].is_ascii_digit() {
        // Example: "5:hello" -> "hello"
        let colon_index = encoded_value.iter().position(|&b| b == b':').unwrap();

        let length = std::str::from_utf8(&encoded_value[..colon_index])
            .unwrap()
            .parse::<usize>()
            .unwrap();

        let string_bytes = &encoded_value[colon_index + 1..colon_index + length+1];

        // Try decoding as UTF-8
        if let Ok(string) = std::str::from_utf8(string_bytes) {
            return (Value::String(string.to_string()), colon_index + length+1);
        } else {
            // Return as byte array if it's not valid UTF-8
            return (
                Value::Array(
                    string_bytes.iter().map(|&b| Value::Number(b.into())).collect(),
                ),
                colon_index + length + 1,
            );
        }
    }
    else if encoded_value[0] == b'i'{
        let end_index = encoded_value.iter().position(|&b| b == b'e').unwrap();

        let number_string = std::str::from_utf8(&encoded_value[1..end_index]).unwrap();

        let number = serde_json::Number::from(number_string.parse::<i64>().unwrap());
        
        return (Value::Number(number) , end_index + 1);
    }
    else if encoded_value[0] == b'l'{
        let mut ptr:usize = 1;
        let mut elements:Vec<Value> = Vec::new();

        while encoded_value[ptr] != b'e'{
            let (element , next_ptr) = decode_bencoded_value(&encoded_value[ptr..]);
            ptr += next_ptr;
            elements.push(element);
        }

        return (Value::Array(elements) , ptr + 1);

    }
    else if encoded_value[0] == b'd' {
        let mut dict: serde_json::Map<String, Value> = serde_json::Map::new();
        let mut ptr: usize = 1; // Start after 'd'
    
        while encoded_value[ptr] != b'e' {
            // Decode the key (must be a string)
            let (key, key_len) = decode_bencoded_value(&encoded_value[ptr..]);
            ptr += key_len;

            
    
            // Ensure that the key is a string
            if let Value::String(key_string) = key {
                // Decode the value associated with the key
                let (value, value_len) = decode_bencoded_value(&encoded_value[ptr..]);
                dict.insert(key_string, value);
                ptr += value_len;
            } else {
                panic!("Dictionary keys must be strings");
            }
        }
    
        // Return the dictionary (in serde_json format) and the number of bytes consumed
        return (Value::Object(dict), ptr + 1); // +1 for the 'e'
    }
    else {
        panic!("Unhandled encoded value: {:?}", encoded_value)
    }
}



fn parse_torrent(file: &str) -> TorrentInfo {
    let torrent_data = fs::read(file).expect("failed to read torrent file");

    let (decoded_dict, _) = decode_bencoded_value(&torrent_data);

    let mut torrent_info: TorrentInfo = Default::default();
    let mut hasher = Sha1::new();

    if let Value::Object(dict) = decoded_dict {
        if let Some(Value::String(announce)) = dict.get("announce") {
            torrent_info.url = announce.to_string();
        } else {
            println!("Tracker URL not found!");
        }

        // Locate the raw bencoded info dictionary
        if let Some(info_start) = torrent_data.windows(6).position(|w| w == b"4:info") {
            let info_bytes = &torrent_data[info_start + 6..];
        
            // Search for the end of the bencoded info dictionary (find the closing 'e')
            let (info_value, info_end) = decode_bencoded_value(info_bytes);
        
            // Hash the raw bencoded 'info' section (the bytes themselves, not the decoded value)
            hasher.update(&torrent_data[info_start + 6..info_start + 6 + info_end]);
            let info_hash = hasher.finalize().to_vec();
            torrent_info.hash = info_hash;
        
            // Proceed with decoding `info_value` for length, piece length, etc.
            if let Value::Object(info_dict) = info_value {
                if let Some(Value::Number(length)) = info_dict.get("length") {
                    torrent_info.length = length.as_i64().unwrap();
                } else {
                    println!("Length field not found!");
                }
        
                if let Some(Value::Number(piece_length)) = info_dict.get("piece length") {
                    torrent_info.piece_length = piece_length.as_i64().unwrap();
                } else {
                    println!("Piece length not found!");
                }
        
                if let Some(Value::Array(pieces)) = info_dict.get("pieces") {
                    let piece_size = 20;
                    for chunk in pieces.chunks(piece_size) {
                        torrent_info.pieces.push(chunk.iter().map(|el| el.as_i64().unwrap() as u8).collect());
                    }
                } else {
                    println!("Pieces not found!");
                }
            }
        } else {
            println!("Info section not found!");
        }


    } else {
        println!("Decoded dictionary not found!");
    }

    torrent_info
}




fn make_tracker_request(torrent_info: &mut TorrentInfo) -> Result<(), Box<dyn std::error::Error>> {

    let info_hash = form_urlencoded::byte_serialize(&torrent_info.hash).collect::<String>();


    let url = format!("{}/?info_hash={}&peer_id={}&port={}&uploaded=0&downloaded=0&left={}&compact=1",
    torrent_info.url , info_hash , torrent_info.peer_id , torrent_info.port , torrent_info.length);

    let res = reqwest::blocking::get(url)?;

    let res_bytes = res.bytes().unwrap();

    let (decoded_res , _) = decode_bencoded_value(&res_bytes);

    if let Some(Value::Array(peers)) = decoded_res.get("peers"){
        for chunk in peers.chunks(6){
            let ip = format!(
                "{}.{}.{}.{}",
                chunk[0].as_i64().unwrap() as u8,
                chunk[1].as_i64().unwrap() as u8,
                chunk[2].as_i64().unwrap() as u8,
                chunk[3].as_i64().unwrap() as u8
            );
    
            // Extract the port number (2 bytes, combine them to form the port).
            let port = ((chunk[4].as_i64().unwrap() as u16) << 8) | (chunk[5].as_i64().unwrap() as u16);
    
            // Print out the peer's IP and port.
            torrent_info.peers_list.push(String::from("{ip}:{port}"));
        }

    }else{
        println!("No peers");
    }

    return Ok(());
}

fn send_handshake(peer_address:&String , torrent_info: &mut TorrentInfo) -> Result<() , Box<dyn std::error::Error>>{
    let mut stream = TcpStream::connect(peer_address).unwrap();
    let mut buffer: Vec<u8> = Vec::with_capacity(68);

    buffer.push(19);
    buffer.extend("BitTorrent protocol".as_bytes());
    buffer.extend(&[0_u8;8]);
    buffer.extend(&torrent_info.hash);
    buffer.extend_from_slice(&torrent_info.peer_id.as_bytes());
    stream.write(&buffer).unwrap();


    stream.read_exact(&mut buffer).unwrap();

    let peer_id = &buffer[48..];

    println!("Peer ID: {}", hex::encode(peer_id));

    Ok(())
}


async fn download_piece(peer_address:&String , torrent_info: &mut TorrentInfo) -> Result<() , Box<dyn std::error::Error>>{
    
    make_tracker_request(torrent_info);
   

    Ok(())
}


// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command:&String = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value: &[u8] = args[2].as_bytes();
        let (decoded_value , _) = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    }
    else if command == "info"{
        let file_path = &args[2];
        let torrent_info = parse_torrent(file_path);
        println!("Tracker URL: {}" , torrent_info.url);
        println!("Length: {}" , torrent_info.length);
        println!("Info Hash: {}" , hex::encode(&torrent_info.hash));
        println!("Piece Length: {}" , torrent_info.piece_length);
        println!("Piece Hashes:");
        for key in torrent_info.pieces{
            println!("{}" , hex::encode(&key));
        }
    }
    else if command == "peers"{
        let file_path = &args[2];
        let mut torrent_info = parse_torrent(file_path);
        torrent_info.port = 6881;
        torrent_info.peer_id = "00112233284566778899".to_string();
    
        if let Err(e) = make_tracker_request(&mut torrent_info) {
            eprintln!("Error making tracker request: {}", e);
        }
        
    }
    else if command == "handshake"{
        let file_path = &args[2];
        let peer_address = &args[3];

        let mut torrent_info = parse_torrent(file_path);
        torrent_info.port = 6881;
        torrent_info.peer_id = "00112233284566778899".to_string();

        if let Err(e) = send_handshake(peer_address, &mut torrent_info){
            println!("Error: {}" , e);
        }
    }
    else if command == "download_piece"{
        let file_path = &args[2];
        let peer_address = &args[3];

        let mut torrent_info = parse_torrent(file_path);
        torrent_info.port = 6881;
        torrent_info.peer_id = "00112233284566778899".to_string();


    }
    else {
        println!("unknown commands lul: {}", args[1])
    }
}
