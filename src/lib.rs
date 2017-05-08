#![cfg_attr(feature = "nightly", feature(raw))]

use std::{str, slice, mem};

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

    unsafe fn assemble_mut(p: *mut u8, len: usize) -> *mut str {
        mem::transmute(str::from_utf8_unchecked(slice::from_raw_parts_mut(p, len)))
    }

    fn disassemble(s: *const str) -> (*const u8, usize) {
        unsafe {
            Referent::disassemble((&*s).as_bytes())
        }
    }

    fn disassemble_mut(s: *mut str) -> (*mut u8, usize) {
        unsafe {
            let bytes: *mut [u8] = mem::transmute((&*s).as_bytes());
            Referent::disassemble_mut(bytes)
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

#[test]
fn test_non_generic_trait_object() {
    trait Foo {
        fn foo(&self) -> String;
    }

    derive_referent!(Foo);

    impl Foo for usize {
        fn foo(&self) -> String {
            format!("{}", self)
        }
    }

    let foo = &(5 as usize) as &Foo;
    let (data, meta) = Referent::disassemble(foo);
    let new_foo: &Foo = unsafe {
        &*Referent::assemble(data, meta)
    };

    assert_eq!(new_foo.foo(), "5");
}

#[test]
fn test_generic_trait_object() {
    use std::borrow::Cow;
    use std::borrow::Cow::*;

    trait Generic<'a, T: Clone> {
        fn value(&'a self) -> Cow<'a, T>;
    }

    derive_referent!(Generic<'a, T>, 'a, T);

    impl<'a> Generic<'a, i32> for i32 {
        fn value(&'a self) -> Cow<'a, i32> {
            Borrowed(self)
        }
    }

    let i = &1 as &Generic<i32>;

    let (data, meta) = Referent::disassemble(i);

    let new_i: &Generic<i32> = unsafe {
        &*Referent::assemble(data, meta)
    };

    assert_eq!(new_i.value(), i.value());

}
