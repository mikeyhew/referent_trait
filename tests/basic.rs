#![cfg(feature = "nightly")]
#[macro_use]
extern crate referent;

trait Foo<'a> {
    fn foo(&'a self) -> &'a str;
}

derive_referent!(Foo<'a>, 'a);

struct Bar;

impl<'a> Foo<'a> for Bar {
    fn foo(&'a self) -> &'static str {
        "Bar!"
    }
}

struct Baz(String);

impl<'a> Foo<'a> for Baz {
    fn foo(&'a self) -> &'a str {
        &self.0
    }
}

#[test]
fn test_reconstruct() {
    use referent::Referent;

    let bar = Bar;
    let baz = Baz(String::from("Baz!"));

    let (data, meta) = Referent::disassemble(&bar as &Foo);
    unsafe {
        let bar_ptr: *const Foo = Referent::assemble(data, meta);
        assert_eq!((*bar_ptr).foo(), bar.foo());
    }

    let (data, meta) = Referent::disassemble(&baz as &Foo);
    unsafe {
        let baz_ptr: *const Foo = Referent::assemble(data, meta);
        assert_eq!((*baz_ptr).foo(), "Baz!");
    }
}

#[test]
fn test_ptr_ext() {
    use referent::{Referent, PtrExt, PtrMutExt};

    let ptr = &mut [1,2,3];
    let (data, meta) = Referent::disassemble_mut(ptr);
    assert_eq!(data, ptr.data());
    assert_eq!(meta, ptr.meta());

    let ptr = "foo";
    let (data, meta) = Referent::disassemble(ptr);
    assert_eq!(data, ptr.data());
    assert_eq!(meta, ptr.meta());
}
