#@ except: DescriptionMissing, InputSorting, LineWidth, MatchingParameterMeta, NonmatchingOutput, RuntimeSectionKeys, Whitespace

version 1.1

task foo {
    meta {}

    parameter_meta{}

    input {
        Int a=- 1
        Int w = 1
        Int x = 2
        Int y = 3
        Int z = 4
        Int f = 5
        Int b = 6
        Int complex_value = w -x +( y* ( z /(f %b) ))
        Boolean complicated_logic = (
            if !(
                a && b || c && (!d || !e)
                == (
                    if foobar
                    then gak
                    else caz
                )
            )
            then "wow"
            else "WOWOWOW"
        )
        Boolean complicated_logic2
            = (
                if
                    !(
                        a
                        && b
                        || c
                        && (
                            !d
                            ||!e
                        )
                        == (
                            if
                                foobar
                            then
                                gak
                            else
                                caz
                        )
                    )
                then
                    "wow"
                else
                    "WOWOWOW"
            )
        Boolean v = if 
        a < b then true 
        else false
        Int k = (  # a comment
            2 * 5
        )
        Boolean l = (  # a comment
            if a < b then true
            else false
        )
        Boolean m = (
            # a comment
            if a < b then true
            else false
        )
        Boolean n = (
            # OK comment
            if a < b  # OK comment
            then true  # OK comment
            else false  # OK comment
        )
    }

    command <<< >>>

    output {
        Boolean b = ! a
    }

    runtime {}
}
