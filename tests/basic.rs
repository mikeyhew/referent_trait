#![cfg(feature = "nightly")]
#[macro_use]
extern crate referent_trait;

trait Foo<'a> {
    fn foo(&self) -> &'a str;
}

derive_referent!(Foo<'a>);

struct Bar;

impl<'a> Foo<'a> for Bar {
    fn foo(&self) -> &'static str {
        "Bar!"
    }
}

struct Baz<'a>(String);

impl<'a> Foo<'a> for Baz<'a> {
    fn foo(&self) -> &'a str {
        &self.0
    }
}

#[test]
fn test_reconstruct() {
    use referent_trait::Referent;

    let bar = Bar;
    let baz1 = Baz(String::from("Baz1!"));
    let baz2 = Baz(String::from("Baz2"));

    let (data, meta) = Referent::disassemble(&bar as &Foo<_>);
    let bar_ptr: *const Foo<_> = Referent::assemble(data, meta);
    assert_eq!(bar_ptr.foo(), bar.foo());
}
