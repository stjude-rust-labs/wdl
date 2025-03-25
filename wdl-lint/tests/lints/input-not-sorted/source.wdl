#@ except: DescriptionMissing, DisallowedInputName, MissingRequirements
#@ except: RuntimeSectionKeys

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
        String a
        File b
        Int c
        Array[Int]? d
        Int? e
        Int? f = 2
        String g = "hello"
        Array[String]? h
        Pair[Int, File] i
        Pair[File, Int] j
        Map[String, Int]? k
        Map[String, Array[Int]]? l
        Map[Int, String]? m
        Pair[String, File] n
        Pair[String, Int] o
        Array[String]+ p
        Array[String] q
        Map[String, File]? r
        Map[String, File] s
        File t
        mystruct u
        #@ except: DeprecatedObject
        Object v
        Directory w
        Directory? x
    }

    output {}
}

#@ except: SectionOrdering
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
