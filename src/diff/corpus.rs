use crate::uci::stockfish::Stockfish;

pub fn edge_case_fens() -> Vec<&'static str> {
    vec![
        "r3k2r/8/8/3Pp3/8/8/8/R3K2R w KQkq e6 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ",
    ]
}

pub fn random_fens(sf: &mut Stockfish, n: usize) -> Vec<String> {
    sf.random_fens(n, 0, 150)
}
