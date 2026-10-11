fn main() {
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).expect("pinned input class reads");
        println!("{}\t{}\t{}", blake3::hash(&bytes).to_hex(), bytes.len(), path);
    }
}
