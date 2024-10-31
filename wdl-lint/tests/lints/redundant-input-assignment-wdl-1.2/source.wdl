#@ except: DescriptionMissing, DisallowedInputName, MissingRequirements
#@ except: RuntimeSectionKeys

version 1.2

workflow test {
    input {
        Int b
        String a
        String c
    }
    # This should flag a note, since version is >= 1.1 and there are redundant input assignments
    # This test was created to ensure the rule works without the explicit "input" keyword in the call statement
    call bar {
         a,  # should not flag
         b = b + 3,  # should not flag
         c = c,  # should flag
   }
}