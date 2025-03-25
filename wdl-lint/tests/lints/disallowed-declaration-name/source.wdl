version 1.2

#@ except: MissingRequirements, SnakeCase, NonmatchingOutput
task test_declaration_names {
    meta {
        description: "This is a test of disallowed declaration names"
    }

    parameter_meta {
        fileInput: "Not OK - has type suffix"
        gtfFile: "Not OK - has type prefix"
        my_int: "Not OK - has type suffix"
        stringValue: "Not OK - has type suffix"
        booleanFlag: "Not OK - has type suffix"
        floatNumber: "Not OK - has type suffix"
        arrayData: "Not OK - has type suffix"
        validName: "OK"
        reference: "OK"
        genome: "OK"
        count: "OK"
    }

    #@ except: InputSorting
    input {
        # Sort the input declarations to match parameter_meta order
        File fileInput
        File gtfFile
        Array[Int] arrayData
        String stringValue
        String genome
        Boolean booleanFlag
        Float floatNumber
        Int my_int
        File validName
        File reference
        Int count
    }

    # Private declarations with type prefixes/suffixes
    File privateFile = "sample.txt"
    Int count_int = 42
    String nameString = "test"

    command <<< >>>

    output {
        # Output declarations with type prefixes/suffixes
        File outputFile = "output.txt"
        Int result_int = 42
        String resultString = "result"
    }

    runtime {}
}
