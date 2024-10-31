#@ except: DescriptionMissing, DisallowedInputName, MissingRequirements
#@ except: RuntimeSectionKeys, MissingOutput, MissingMetas

version 1.1

workflow test1 {
    input {
        String a
        String c
        Int b
    }

    call bar { input:
         a,  # should not flag
         b = b + 3,  # should not flag
         c = c,  # This should flag a note, since version is >= 1.1
   }
}

workflow test2 {
    input {
        String a
        String c
        Int b
    }

    #@ except: RedundantInputAssignment
    call bar { input:
         a,  # should not flag
         b = b + 3,  # should not flag
         c = c,  # This should not flag a note due to the except statement
   }
}
