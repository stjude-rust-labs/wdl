# This is a test of an unterminated metadata string.

version 1.1

task test {
    meta {
        foo: "bar"
        a: 'unterminated
    }
}
