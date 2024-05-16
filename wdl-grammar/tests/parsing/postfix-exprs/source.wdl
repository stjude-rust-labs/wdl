# This is a test of postfix expressions

version 1.1

task test {
    Integer a = min(0, 1)
    Integer b = min(max(100, a), 10)
    Array[String] c = ["a", "b", "c"]
    String d = d[a + b]
    MyStruct e = MyStruct {
        foo: MyFoo {
            bar: "baz",
        }
    }
    MyFoo f = MyFoo {
        foo: e.foo.bar
    }
}