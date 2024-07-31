#@ except: DeprecatedObject, DescriptionMissing, DisallowedInputName, LineWidth, SectionOrdering, RuntimeSectionKeys

version 1.2

#@ except: MissingMetas
struct Mystruct {
    String a
    Int b
}

workflow foo {
    meta {}
    parameter_meta {
        a: ""
        b: ""
        c: ""
        d: ""
        e: ""
        f: ""
        g: ""
        h: ""
        i: ""
        j: ""
        k: ""
        l: ""
        m: ""
        n: ""
        o: ""
        p: ""
        q: ""
        r: ""
        s: ""
        t: ""
        u: ""
        v: ""
        w: ""
        x: ""
    }
    input {
        String g = "hello"
        Int? f = 2
        Int? e
        Int c
        Array[String]? h
        File t
        String a
        Pair[Int, File] i
        File b
        Pair[String, Int] o
        Pair[File, Int] j
        Array[Int]? d
        Array[String] q
        Object v
        Map[String, Int]? k
        Map[String, Array[Int]]? l
        Map[Int, String]? m
        Map[String, File]? r
        Directory w
        Directory? x
        Map[String, File] s
        Pair[String, File] n
        Array[String]+ p
        mystruct u
    }
    output {}
}

task bar {
    meta {}
    parameter_meta {
        a: ""
        b: ""
        c: ""
        d: ""
        e: ""
        f: ""
        g: ""
        h: ""
        i: ""
        j: ""
        k: ""
        l: ""
        m: ""
        n: ""
        o: ""
        p: ""
        q: ""
        r: ""
        s: ""
        t: ""
        w: ""
        x: ""
    }
    input {
        String g = "hello"
        Int? f = 2
        Int? e
        Int c
        Array[String]? h
        File t
        String a
        Pair[Int, File] i
        File b
        Pair[String, Int] o
        Pair[File, Int] j
        Array[Int]? d
        Array[String] q
        Map[String, Int]? k
        Map[String, Array[Int]]? l
        Map[Int, String]? m
        Map[String, File]? r
        Directory w
        Directory? x
        Map[String, File] s
        Pair[String, File] n
        Array[String]+ p
    }
    command <<<
    >>>
    runtime {}
    output {}
}
