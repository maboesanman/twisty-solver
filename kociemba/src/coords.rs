use deku::prelude::*;
use paste::paste;

macro_rules! define_coord {
    ($name:ident, $inner:ty, $range:expr, $bits:expr) => {
        #[derive(Debug, PartialEq, DekuRead, DekuWrite, Copy, Clone)]
        #[deku(endian = "big")]
        pub struct $name (
            #[deku(bits = $bits)]
            $inner
        );

        impl const Into<$inner> for $name {
            fn into(self) -> $inner {
                self.0
            }
        }
        
        impl const From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }
    }
}

macro_rules! define_raw_coord_no_move {
    ($name:ident, $inner:ty, $range:expr, $bits:expr) => {
        paste! {
            define_coord!([<$name Coord>], $inner, $range, $bits);

            pub type [<$name MoveTable>] = [[<$name MoveEntry>]; $range];

            #[derive(Clone, Copy)]
            pub struct [<$name MoveEntry>] {
                pub transforms: [[<$name Coord>]; 15],
            }
        }
    }
}

macro_rules! define_raw_coord {
    ($name:ident, $inner:ty, $range:expr, $bits:expr) => {
        paste! {
            define_coord!([<$name Coord>], $inner, $range, $bits);

            pub type [<$name MoveTable>] = [[<$name MoveEntry>]; $range];

            #[derive(Clone, Copy)]
            pub struct [<$name MoveEntry>] {
                pub transforms: [[<$name Coord>]; 15],
                pub moves: [[<$name Coord>]; 18],
            }
        }
    }
}

macro_rules! define_sym_coord {
    ($name:ident, $inner:ty, $range:expr, $bits:expr, $moves:expr) => {
        paste! {
            define_coord!([<$name SymCoord>], $inner, $range, $bits);

            pub type [<$name SymMoveTable>] = [[<$name SymMoveEntry>]; $range];

            pub struct [<$name SymMoveEntry>] {
                pub moves: [([<$name SymCoord>], Transform); $moves],
            }
        }
    }
}

// transforms

define_coord!(Transform, u8, 16, 4);

// raw coordinates

// 2187 (11.09 bits)
// tables: (866,052 bits, 108kb)
// - all moves
// - all transforms
define_raw_coord!(CornerOrient, u16, 2187, 12);

// 2048 (11 bits) u16
// tables: (743,424 bits, 93kb)
// - all moves
// - all transforms
define_raw_coord_no_move!(EdgeOrient, u16, 2048, 11);

// 495 (8.9 bits) u16
// tables: (147,015 bits, 18kb)
// - all moves
// - all transforms
define_raw_coord_no_move!(EdgeGroup, u16, 495, 9);

// 40320 (15.29 bits) u16
// tables: (21,288,960 bits, 2,661kb)
// - all moves
// - all transforms
define_raw_coord!(CornerPerm, u16, 40320, 16);

// 40320 (15.29 bits) u16
// tables: (21,288,960 bits, 2,661kb)
// - all moves
// - all transforms
define_raw_coord!(UDEdgePerm, u16, 40320, 16);

// 24 (4.58 bits) u8
// tables: (3600 bits, 450 bytes)
// - all moves
// - all transforms
define_raw_coord!(EEdgePerm, u8, 24, 5);

// sym coordinates

// 64430 (15.97 bits) u16
// tables: (23,194,800 bits, 2,899kb)
// - all moves + transform index
define_sym_coord!(EdgeOrientGroup, u16, 64430, 16, 18);

// 2768 (11.43 bits) u16
// tables: (442,880 bits, 55kb)
// - all moves + transform index
define_sym_coord!(CornerPerm, u16, 2768, 12, 10);

// total table size: 8.5 mb
