#@ except: DescriptionMissing, MissingRequirements
#@ except: RuntimeSectionKeys, MissingOutput, MissingMetas

version 1.0

workflow test {
    input {
        String a
        String c
        Int b
    }

    # This should not flag any notes, since version is 1.0
    call bar { input:
         a.
         b = b + 3,
         c = c,
   }
}
