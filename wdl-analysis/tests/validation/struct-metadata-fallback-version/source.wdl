## This is a test of struct metadata sections in a WDL document with an unrecognized version. The
## analysis is configured to fall back to version 1.2 upon encountering an unrecognized version,
## which is the minimum required for struct metadata, and so this should validate.

version devel

struct Foo {
    Int a

    meta {
        foo: "bar"
    }

    parameter_meta {
        a: "foo"
        b: "bar"
    }

    String b
}
