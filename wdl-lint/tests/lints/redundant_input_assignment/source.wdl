#@ except: DescriptionMissing, DisallowedInputName, MissingRequirements
#@ except: RuntimeSectionKeys

version 1.2

#@ except: SectionOrdering
task bar {
    meta {}

    parameter_meta {
        a: ""
        b: 2
        c: ""
    }

    input {
        int b
        String a = a
        String c = c
    }

    command <<<
    >>>

    runtime {}

    output {}
}
