## This is a test of having duplicate struct names in a document.

version 1.1

import "foo" alias Foo as Baz

struct Foo {
    Int x
}

struct Bar {
    Int x
}

struct Foo {
    Int x
}

struct Bar {
    Int x
}

struct Baz {
    Int x
}

import "Bar" alias Baz as Foo
