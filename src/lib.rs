#![cfg_attr(feature = "nightly", feature(raw))]

/// A trait that is implemented for all types except trait objects, for which it can be derived with the `derive_referent!` macro on nightly. Provides functions to split apart/reconstruct a fat pointer into/from its components.
pub trait Referent {
    type Data;
    type Meta: Copy;

    /// Make a reference from its constituent parts.
    unsafe fn assemble(data: *const Self::Data, meta: Self::Meta) -> *const Self;

    unsafe fn assemble_mut(data: *mut Self::Data, meta: Self::Meta) -> *mut Self;

    /// Break a reference down into its constituent parts.
    fn disassemble(fatp: *const Self) -> (*const Self::Data, Self::Meta);

    fn disassemble_mut(fatp: *mut Self) -> (*mut Self::Data, Self::Meta);
}

impl<T> Referent for T {
    type Data = T;
    type Meta = ();

    unsafe fn assemble(p: *const T, _: ()) -> *const T {
        p
    }

    unsafe fn assemble_mut(p: *mut T, _: ()) -> *mut T {
        p
    }

    fn disassemble(p: *const T) -> (*const T, ()) {
        (p, ())
    }

    fn disassemble_mut(p: *mut T) -> (*mut T, ()) {
        (p, ())
    }
}

use std::slice;

impl<T> Referent for [T] {
    type Data = T;
    type Meta = usize;

    unsafe fn assemble(p: *const T, len: usize) -> *const [T] {
        slice::from_raw_parts(p, len)
    }

    unsafe fn assemble_mut(p: *mut T, len: usize) -> *mut [T] {
        slice::from_raw_parts_mut(p, len)
    }

    fn disassemble(slice: *const [T]) -> (*const T, usize) {
        let slice = unsafe { &*slice };
        (slice.as_ptr(), slice.len())
    }

    fn disassemble_mut(slice: *mut [T]) -> (*mut T, usize) {
        let slice = unsafe { &mut *slice };
        (slice.as_mut_ptr(), slice.len())
    }
}

#[test]
fn test_slice() {
    let slice = &[1,2,3];
    let (ptr, len) = Referent::disassemble(slice);
    let new_slice: &[i32] = unsafe {
        &*Referent::assemble(ptr, len)
    };
    assert_eq!(new_slice, slice);
}

#[cfg(feature = "nightly")]
#[macro_export]
/**

*/
macro_rules! derive_referent {
    ($Trait:ty) => {
        impl $crate::Referent for $Trait {
            type Data = ();
            type Meta = $crate::nightly::Meta;

            unsafe fn assemble(data: *const Self::Data, meta: Self::Meta) -> *const Self {
                ::std::mem::transmute(
                    $crate::nightly::TraitObject::construct(data as *mut (), meta)
                )
            }

            unsafe fn assemble_mut(data: *mut Self::Data, meta: Self::Meta) -> *mut Self {
                ::std::mem::transmute(
                    $crate::nightly::TraitObject::construct(data, meta)
                )
            }

            fn disassemble(fatp: *const Self) -> (*const Self::Data, Self::Meta) {
                let trait_object: $crate::nightly::TraitObject = unsafe {
                    ::std::mem::transmute(fatp)
                };

                (trait_object.data(), trait_object.meta())
            }

            fn disassemble_mut(fatp: *mut Self) -> (*mut Self::Data, Self::Meta) {
                let trait_object: $crate::nightly::TraitObject = unsafe {
                    ::std::mem::transmute(fatp)
                };

                (trait_object.data(), trait_object.meta())
            }
        }
    }
}

#[cfg(feature = "nightly")]
#[doc(hidden)]
pub mod nightly {
    use std::raw;

    #[derive(Copy, Clone)]
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

// #[cfg(test)]
// mod tests {
//     use super::Referent;
//
//     trait MyTrait {}
//     derive_referent!(MyTrait);
//
//     impl<T: ?Sized> MyTrait for T {}
//
//     #[test]
//     fn test_basic() {
//         let foo = "0123456789";
//         let (data, meta) = Referent::disassemble(foo);
//         unsafe {
//             *Referent::assemble(data, meta);
//         }
//     }
// }
