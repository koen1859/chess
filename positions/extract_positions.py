import chess
import chess.pgn
import chess.engine
import argparse
import sys
from pathlib import Path


def count_non_pawn_pieces(board):
    count = 0
    for color in (chess.WHITE, chess.BLACK):
        for piece_type in (chess.QUEEN, chess.ROOK, chess.BISHOP, chess.KNIGHT):
            count += len(board.pieces(piece_type, color))
    return count


def extract_position(game, min_ply, max_ply, min_pieces):
    board = game.board()
    best_fen = None
    best_pieces = 0
    for i, move in enumerate(game.mainline_moves()):
        board.push(move)
        ply = i + 1
        if ply < min_ply:
            continue
        if ply > max_ply:
            break
        pieces = count_non_pawn_pieces(board)
        if pieces >= min_pieces and pieces > best_pieces:
            best_fen = board.fen()
            best_pieces = pieces
    return best_fen


def filter_positions(fens, engine_path, depth, threshold):
    engine = chess.engine.SimpleEngine.popen_uci(engine_path)
    try:
        engine.configure({"Hash": 256})
        filtered = []
        total = len(fens)
        for idx, fen in enumerate(fens):
            board = chess.Board(fen)
            result = engine.analyse(board, chess.engine.Limit(depth=depth))
            score = result["score"].white().score(mate_score=100000)
            if score is not None:
                cp = score / 100.0
                if abs(cp) <= threshold:
                    filtered.append((fen, cp))
            if (idx + 1) % 100 == 0:
                print(
                    f"  Stockfish: {idx + 1}/{total} analyzed, {len(filtered)} kept",
                    file=sys.stderr,
                )
    finally:
        engine.quit()
    return filtered


def main():
    parser = argparse.ArgumentParser(
        description="Extract balanced middlegame positions from a PGN file."
    )
    parser.add_argument("pgn", help="Input PGN file")
    parser.add_argument("--min-ply", type=int, default=20)
    parser.add_argument("--max-ply", type=int, default=60)
    parser.add_argument("--min-pieces", type=int, default=10)
    parser.add_argument("--depth", type=int, default=18)
    parser.add_argument("--threshold", type=float, default=0.5)
    parser.add_argument("--output", default="balanced.epd")
    parser.add_argument("--stockfish", default="stockfish")
    parser.add_argument(
        "--max-games",
        type=int,
        default=0,
        help="Max games to process from PGN (0 = all)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=0,
        help="Max positions before Stockfish filtering (0 = unlimited)",
    )
    args = parser.parse_args()

    pgn_path = Path(args.pgn)
    if not pgn_path.exists():
        print(f"Error: {pgn_path} not found", file=sys.stderr)
        sys.exit(1)

    # Phase 1: extract positions from PGN
    print(f"Extracting positions from {pgn_path}...", file=sys.stderr)
    positions = []
    game_count = 0
    with open(pgn_path) as f:
        while True:
            game = chess.pgn.read_game(f)
            if game is None:
                break
            game_count += 1
            fen = extract_position(game, args.min_ply, args.max_ply, args.min_pieces)
            if fen:
                positions.append(fen)
            if game_count % 10000 == 0:
                print(
                    f"  Games: {game_count}, positions found: {len(positions)}",
                    file=sys.stderr,
                )
            if args.max_games and game_count >= args.max_games:
                break

    print(
        f"Processed {game_count} games, extracted {len(positions)} positions",
        file=sys.stderr,
    )

    if not positions:
        print(
            "No positions found. Try lowering --min-pieces or widening ply range.",
            file=sys.stderr,
        )
        sys.exit(1)

    # Phase 2: filter with Stockfish
    if args.limit > 0 and len(positions) > args.limit:
        positions = positions[: args.limit]

    print(
        f"Filtering {len(positions)} positions with Stockfish "
        f"(depth {args.depth}, |eval| <= {args.threshold})...",
        file=sys.stderr,
    )
    filtered = filter_positions(positions, args.stockfish, args.depth, args.threshold)

    # Phase 3: write output
    output_path = Path(args.output)
    with open(output_path, "w") as f:
        for fen, score in filtered:
            f.write(f"{fen}\n")

    print(f"Wrote {len(filtered)} balanced positions to {output_path}", file=sys.stderr)
    if filtered:
        scores = [s for _, s in filtered]
        print(
            f"  Score range: [{min(scores):.2f}, {max(scores):.2f}], "
            f"avg: {sum(scores) / len(scores):.2f}",
            file=sys.stderr,
        )


if __name__ == "__main__":
    main()
