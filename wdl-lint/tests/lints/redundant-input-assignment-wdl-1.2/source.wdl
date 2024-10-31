#@ except: DescriptionMissing, DisallowedInputName, MissingRequirements
#@ except: RuntimeSectionKeys, MissingMetas, MissingOutput

version 1.2

workflow test1 {
    input {
        String a
        String c
        Int b
    }

    # This should flag, since version >= 1.1 and there are redundant input assignments
    # This test was created to ensure the rule works without the explicit "input"
    call bar {
         a,  # should not flag
         b = b + 3,  # should not flag
         c = c,  # should flag
   }
}

workflow test2 {
    input {
        String a
        String c
        Int b
    }

    # Should flag since version >= 1.1 and there are redundant input assignments
    # This test was created to ensure the rule works without explicit "input" keyword
    #@ except: RedundantInputAssignment
    call bar {
         a,  # should not flag
         b = b + 3,  # should not flag
         c = c,  # should flag
   }
}