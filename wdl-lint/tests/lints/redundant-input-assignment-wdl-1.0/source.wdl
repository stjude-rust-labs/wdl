#@ except: DescriptionMissing, DisallowedInputName, MissingRequirements
#@ except: RuntimeSectionKeys

version 1.0

workflow test {

    input {
        Int b
        String a
        String c
    }
    # This should not flag any notes, since version is 1.0
    call bar { input:
         a,  # should not flag
         b = b + 3,  # should not flag
         c = c,  # should flag
   }
}
