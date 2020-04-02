use hermes::internal::bencoding;

fn main() {
    println!("hermes - BitTorrent client");

    let decoder = bencoding::Decoder::new("test.torrent");
    let dictionary = decoder.decode();
}
