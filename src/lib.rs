#![no_std]
#![cfg_attr(feature = "nightly", feature(raw))]

use core::{str, slice, ptr};

#[doc(hidden)]
pub use core::mem;

/// A trait that is implemented for all types except trait objects, for which it can be derived with the `derive_referent!` macro on nightly. Provides functions to split apart/reconstruct a fat pointer into/from its components.
pub trait Referent {
    type Data;
    type Meta: Copy;

    /// Make a reference from its constituent parts.
    unsafe fn assemble(data: *const Self::Data, meta: Self::Meta) -> *const Self;

    unsafe fn assemble_mut(data: *mut Self::Data, meta: Self::Meta) -> *mut Self {
        mem::transmute(Self::assemble(data, meta))
    }

    /// Break a reference down into its constituent parts.
    fn disassemble(fatp: *const Self) -> (*const Self::Data, Self::Meta);

    fn disassemble_mut(fatp: *mut Self) -> (*mut Self::Data, Self::Meta) {
        let (data, meta) = Self::disassemble(fatp);
        unsafe {
            (mem::transmute(data), meta)
        }
    }

    fn size_of_val(meta: Self::Meta) -> usize {
        let r = unsafe {
            &*Self::assemble(ptr::null(), meta)
        };

        mem::size_of_val(r)
    }

    fn align_of_val(meta: Self::Meta) -> usize {
        let r = unsafe {
            &*Self::assemble(ptr::null(), meta)
        };

        mem::align_of_val(r)
    }
}

impl<T> Referent for T {
    type Data = T;
    type Meta = ();

    unsafe fn assemble(p: *const T, _: ()) -> *const T {
        p
    }

    fn disassemble(p: *const T) -> (*const T, ()) {
        (p, ())
    }
}

impl<T> Referent for [T] {
    type Data = T;
    type Meta = usize;

    unsafe fn assemble(p: *const T, len: usize) -> *const [T] {
        slice::from_raw_parts(p, len)
    }

    fn disassemble(slice: *const [T]) -> (*const T, usize) {
        let slice = unsafe { &*slice };
        (slice.as_ptr(), slice.len())
    }
}

#[test]
fn test_slice() {
    let slice = &[1,2,3] as &[i32];
    let (ptr, len) = Referent::disassemble(slice);
    let new_slice: &[i32] = unsafe {
        &*Referent::assemble(ptr, len)
    };
    assert_eq!(new_slice, slice);
}

impl Referent for str {
    type Data = u8;
    type Meta = usize;

    unsafe fn assemble(p: *const u8, len: usize) -> *const str {
        str::from_utf8_unchecked(slice::from_raw_parts(p, len))
    }

    fn disassemble(s: *const str) -> (*const u8, usize) {
        unsafe {
            Referent::disassemble((&*s).as_bytes())
        }
    }
}

#[test]
fn test_str() {
    let s = "Yolo123";
    let (ptr, len) = Referent::disassemble(s);
    let new_s: &str = unsafe {
        &*Referent::assemble(ptr, len)
    };
    assert_eq!(s, new_s);
}

#[cfg(feature = "nightly")]
#[macro_export]
#[doc(hidden)]
macro_rules! __derive_referent_body {
    ($Trait:ty) => {
        type Data = ();
        type Meta = $crate::nightly::Meta;

        unsafe fn assemble(data: *const Self::Data, meta: Self::Meta) -> *const Self {
            $crate::mem::transmute(
                $crate::nightly::TraitObject::construct(data as *mut (), meta)
            )
        }

        fn disassemble(fatp: *const Self) -> (*const Self::Data, Self::Meta) {
            let trait_object: $crate::nightly::TraitObject = unsafe {
                $crate::mem::transmute(fatp)
            };

            (trait_object.data(), trait_object.meta())
        }
    };
}

#[cfg(feature = "nightly")]
#[macro_export]
macro_rules! derive_referent {
    ($Trait:ty) => {
        impl $crate::Referent for $Trait {
            __derive_referent_body!($Trait);
        }
    };

    ($Trait:ty, $($args:tt),+ ) => {
        impl<$($args),+> $crate::Referent for $Trait {
            __derive_referent_body!($Trait);
        }
    };
}

#[cfg(feature = "nightly")]
#[doc(hidden)]
pub mod nightly {
    use core::raw;

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Meta(*mut ());

    #[derive(Copy, Clone)]
    pub struct TraitObject(raw::TraitObject);

    impl TraitObject {
        pub fn construct(data: *mut (), meta: Meta) -> TraitObject {
            TraitObject(raw::TraitObject {
                data: data,
                vtable: meta.0,
            })
        }

        pub fn data(self) -> *mut () {
            self.0.data
        }

        pub fn meta(self) -> Meta {
            Meta(self.0.vtable)
        }
    }
}

pub trait PtrExt {
    type Referred: Referent + ?Sized;

    fn meta(self) -> <Self::Referred as Referent>::Meta;
}

impl<T: Referent + ?Sized> PtrExt for *const T {
    type Referred = T;

    fn meta(self) -> T::Meta  {
        let (_, meta) = T::disassemble(self);
        meta
    }
}

impl<'a, T: Referent + ?Sized + 'a> PtrExt for &'a T {
    type Referred = T;

    fn meta(self) -> T::Meta  {
        (self as *const T).meta()
    }
}

impl<T: Referent + ?Sized> PtrExt for *mut T {
    type Referred = T;

    fn meta(self) -> T::Meta  {
        (self as *const T).meta()
    }
}

impl<'a, T: Referent + ?Sized + 'a> PtrExt for &'a mut T {
    type Referred = T;

    fn meta(self) -> T::Meta  {
        (self as *const T).meta()
    }
}
