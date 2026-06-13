use crate::chess::{chess::Chess, square::Square};
use bitflags::bitflags;

bitflags! {
   #[derive(Clone, Copy)]
   pub struct CastlingRights: u8 {
        const NONE = 0;
        const WHITEKINGSIDE=1<<0;
        const WHITEQUEENSIDE=1<<1;
        const BLACKKINGSIDE=1<<2;
        const BLACKQUEENSIDE=1<<3;
        const ALL = Self::WHITEKINGSIDE.bits()
            |Self::WHITEQUEENSIDE.bits()
            |Self::BLACKKINGSIDE.bits()
            |Self::BLACKQUEENSIDE.bits();
    }
}

impl Chess {
    pub fn update_castling_rights(&mut self, from: usize, piece: Square) {
        match piece {
            Square::WhiteKing => {
                self.castling_rights
                    .remove(CastlingRights::WHITEKINGSIDE | CastlingRights::WHITEQUEENSIDE);
            }

            Square::BlackKing => {
                self.castling_rights
                    .remove(CastlingRights::BLACKKINGSIDE | CastlingRights::BLACKQUEENSIDE);
            }

            Square::WhiteRook => {
                match from {
                    0 => self.castling_rights.remove(CastlingRights::WHITEQUEENSIDE), // a1
                    7 => self.castling_rights.remove(CastlingRights::WHITEKINGSIDE),  // h1
                    _ => {}
                }
            }

            Square::BlackRook => {
                match from {
                    56 => self.castling_rights.remove(CastlingRights::BLACKQUEENSIDE), // a8
                    63 => self.castling_rights.remove(CastlingRights::BLACKKINGSIDE),  // h8
                    _ => {}
                }
            }

            _ => {}
        }
    }
}
