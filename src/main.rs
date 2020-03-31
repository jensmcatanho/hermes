use hermes::internal::bencoding;

fn main() {
    println!("hermes - BitTorrent client");

    println!("{}", bencoding::Decoder::decode("filename"));
}
