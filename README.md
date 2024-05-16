# rustychess (TODO: Come up with a fun name)

A chess library (WIP)

Implemented:
Board state:
  - uses 12 bitboards to represent pieces
  - keeps track of en passant square, turn count, occupied squares, and castling rights
Move generation:
  - pawn moves, double moves, attacks, and ep captures.
  - knight moves

TODO:
  - generate king moves from lookup table
  - generate sliding piece attacks in some clever way
  - design ui using some framework (TBD)
  - implement magic bitboards
  - create a 1500 elo bot using the engine.

Based on Stockfish, Pleco, and chess programming wiki.
https://github.com/official-stockfish/Stockfish
https://github.com/pleco-rs/Pleco
https://www.chessprogramming.org/Main_Page
