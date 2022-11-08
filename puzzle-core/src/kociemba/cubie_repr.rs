use std::intrinsics::ptr_offset_from;

use memoffset::offset_of;
use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct CubieRepr {
    // THE ORIENTATION HERE IS IMPORTANT
    // EDGE_ORIENT AND CORNER ORIENT ARE SUPPOSED TO BE ADJACENT
    // see corner resident enum for ordering
    pub(crate) corner_perm: [CornerResident; 8],
    pub(crate) corner_orient: [CornerOrient; 8],

    // see edge resident enum for ordering
    // only first 12 are used
    pub(crate) edge_orient: [EdgeOrient; 12],
    pub(crate) edge_perm: [EdgeResident; 12],
}

#[test]
fn layout_correct() {
    assert_eq!(std::mem::size_of::<CubieRepr>(), 40);
    assert_eq!(offset_of!(CubieRepr, corner_perm), 0);
    assert_eq!(offset_of!(CubieRepr, corner_orient), 8);
    assert_eq!(offset_of!(CubieRepr, edge_orient), 16);
    assert_eq!(offset_of!(CubieRepr, edge_perm), 28);
}

pub(crate) const fn corner_perm_offset() -> usize {
    let c = CubieRepr::new();
    let c_ptr: *const _ = &c;
    let c_c_p_ptr: *const _ = &c.corner_perm;

    unsafe { ptr_offset_from(c_c_p_ptr as *const u8, c_ptr as *const u8) as usize }
}

pub(crate) const fn corner_orient_offset() -> usize {
    let c = CubieRepr::new();
    let c_ptr: *const _ = &c;
    let c_c_o_ptr: *const _ = &c.corner_orient;

    unsafe { ptr_offset_from(c_c_o_ptr as *const u8, c_ptr as *const u8) as usize }
}

pub(crate) const fn edge_perm_offset() -> usize {
    let c = CubieRepr::new();
    let c_ptr: *const _ = &c;
    let c_e_p_ptr: *const _ = &c.edge_perm;

    unsafe { ptr_offset_from(c_e_p_ptr as *const u8, c_ptr as *const u8) as usize }
}

pub(crate) const fn edge_orient_offset() -> usize {
    let c = CubieRepr::new();
    let c_ptr: *const _ = &c;
    let c_e_o_ptr: *const _ = &c.edge_orient;

    unsafe { ptr_offset_from(c_e_o_ptr as *const u8, c_ptr as *const u8) as usize }
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    UnsafeFromPrimitive,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[repr(u8)]
pub(crate) enum CornerResident {
    UFR = 0,
    UFL = 1,
    UBR = 2,
    UBL = 3,
    DFR = 4,
    DFL = 5,
    DBR = 6,
    DBL = 7,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, UnsafeFromPrimitive, TryFromPrimitive, IntoPrimitive)]
pub(crate) enum CornerOrient {
    Solved = 0,
    Clockwise = 1,
    CounterClockwise = 2,
}

#[repr(u8)]
#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    UnsafeFromPrimitive,
    TryFromPrimitive,
    IntoPrimitive,
)]
pub(crate) enum EdgeResident {
    UF = 0,
    UB = 1,
    DF = 2,
    DB = 3,
    FR = 4,
    FL = 5,
    BR = 6,
    BL = 7,
    UR = 8,
    UL = 9,
    DR = 10,
    DL = 11,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, UnsafeFromPrimitive, TryFromPrimitive, IntoPrimitive)]
pub(crate) enum EdgeOrient {
    Solved = 0,
    Unsolved = 1,
}

impl Default for CubieRepr {
    fn default() -> Self {
        Self::new()
    }
}

impl CubieRepr {
    pub const fn new() -> Self {
        Self {
            corner_perm: [
                CornerResident::UFR,
                CornerResident::UFL,
                CornerResident::UBR,
                CornerResident::UBL,
                CornerResident::DFR,
                CornerResident::DFL,
                CornerResident::DBR,
                CornerResident::DBL,
            ],
            corner_orient: [
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
                CornerOrient::Solved,
            ],
            edge_perm: [
                EdgeResident::UF,
                EdgeResident::UB,
                EdgeResident::DF,
                EdgeResident::DB,
                EdgeResident::FR,
                EdgeResident::FL,
                EdgeResident::BR,
                EdgeResident::BL,
                EdgeResident::UR,
                EdgeResident::UL,
                EdgeResident::DR,
                EdgeResident::DL,
            ],
            edge_orient: [
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
                EdgeOrient::Solved,
            ],
        }
    }
    pub(crate) const fn into_array(self) -> [u8; 40] {
        unsafe { core::mem::transmute(self) }
    }
    pub(crate) const unsafe fn from_array_unchecked(array: [u8; 40]) -> Self {
        unsafe { core::mem::transmute(array) }
    }
    pub(crate) const fn into_ref(&self) -> &[u8; 40] {
        unsafe { core::mem::transmute(self) }
    }
    pub(crate) fn into_mut_ref(&mut self) -> &mut [u8; 40] {
        unsafe { core::mem::transmute(self) }
    }

    pub(crate) fn is_valid(&self) -> bool {
        let mut v = self
            .corner_perm
            .iter()
            .map(|x| *x as u8)
            .collect::<Vec<_>>();
        v.sort();
        if v != (0..8u8).into_iter().collect::<Vec<_>>() {
            return false;
        }

        let mut v = self.edge_perm.iter().map(|x| *x as u8).collect::<Vec<_>>();
        v.sort();
        if v != (0..12u8).into_iter().collect::<Vec<_>>() {
            return false;
        }

        true
    }

    pub fn is_solved(&self) -> bool {
        self == &CubieRepr::new()
    }
}
