#@ except: DescriptionMissing, DisallowedInputName, MissingRequirements
#@ except: RuntimeSectionKeys

version 1.1

workflow test {
    input {
        Int b
        String a
        String c
    }

    call bar { input:
         a,  # should not flag
         b = b + 3,  # should not flag
         c = c, # This should flag a note, since version is >= 1.1
   }
}