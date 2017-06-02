#[macro_use] extern crate referent;

use referent::{Referent, PtrExt};

use std::mem;

trait Foo {}
impl<T: ?Sized> Foo for T {}
derive_referent!(Foo);

fn check<T: Referent + ?Sized>(r: &T) {
    assert_eq!(
        T::size_of_val(r.meta()),
        mem::size_of_val(r)
    );

    assert_eq!(
        T::align_of_val(r.meta()),
        mem::align_of_val(r)
    );
}

#[test]
fn test_same_size_and_align() {
    check("foo");
    check(&"foo");
    check(&[1,2,3]);
    check(&String::from("yolo"));
    check(&1);

    check(&"foo" as &Foo);
    check(&[1,2,3] as &Foo);
    check(&1 as &Foo);
}
