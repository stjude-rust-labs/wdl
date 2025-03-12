## This is a test of passing complex type inputs using YAML format.
## No error should be present in `error.txt`.
## This test uses inputs.yaml instead of inputs.json.

version 1.2

struct Foo {
    Int foo
    String bar
    Bar baz
}

struct Bar {
    File foo
    Directory bar
    Baz baz
}

struct Baz {
    Boolean foo
    Float bar
}

task foo {
    input {
        Foo foo
    }

    command <<<>>>
}

workflow test {
    meta {
        allowNestedInputs: true
    }

    input {
        Foo foo
        Bar bar
        Baz baz
        Int? x
        Array[Float] y
    }

    call foo as my_call
}
