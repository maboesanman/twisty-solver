use deku::prelude::*;

macro_rules! define_coord {
    ($name:ident, $inner:ty, $bits:expr) => {
        #[derive(Debug, PartialEq, DekuRead, DekuWrite, Copy, Clone)]
        #[deku(endian = "big")]
        pub struct $name(
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

// raw coordinates

// 2187 (11.09 bits)
// tables: (866,052 bits, 108kb)
// - all moves
// - all transforms
define_coord!(CornerOrientCoord, u16, 12);

// 2048 (11 bits) u16
// tables: (743,424 bits, 93kb)
// - all moves
// - all transforms
define_coord!(EdgeOrientCoord, u16, 11);

// 495 (8.9 bits) u16
// tables: (147,015 bits, 18kb)
// - all moves
// - all transforms
define_coord!(EdgeGroupCoord, u16, 9);

// 40320 (15.29 bits) u16
// tables: (21,288,960 bits, 2,661kb)
// - all moves
// - all transforms
define_coord!(CornerPermCoord, u16, 16);

// 40320 (15.29 bits) u16
// tables: (21,288,960 bits, 2,661kb)
// - all moves
// - all transforms
define_coord!(UDEdgePermCoord, u16, 16);

// 24 (4.58 bits) u8
// tables: (3600 bits, 450 bytes)
// - all moves
// - all transforms
define_coord!(EEdgePermCoord, u8, 5);

// sym coordinates

// 64430 (15.97 bits) u16
// tables: (23,194,800 bits, 2,899kb)
// - all moves + transform index
define_coord!(EdgeOrientGroupSymCoord, u16, 16);

// 2768 (11.43 bits) u16
// tables: (442,880 bits, 55kb)
// - all moves + transform index
define_coord!(CornerPermSymCoord, u16, 12);

// total table size: 8.5 mb
