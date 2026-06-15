use bitflags::bitflags;

bitflags! {
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
