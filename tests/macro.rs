#![cfg(feature = "nightly")]
#[macro_use]
extern crate referent;

use referent::Referent;

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
