## This is a test of unnecessary function calls.
#@ except: UnusedDeclaration

version 1.1

workflow test {
    String foo = select_first(['foo', 'bar', 'baz'])
    Array[String] bar = select_all(['foo', 'bar', 'baz'])
    Boolean baz = defined(['foo', 'bar', 'baz'])
}
