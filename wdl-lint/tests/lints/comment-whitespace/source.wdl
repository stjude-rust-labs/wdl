#@ except: DescriptionMissing, MissingMetas, NonmatchingOutput, Whitespace
## some comment

version 1.2


################

#a bad comment
    # another bad comment

# a good comment

workflow foo {# test in-line comment without preceding whitespace
    meta {# this is a problematic yet valid comment
    }

    input { # a bad comment
        String foo  # a good comment
    # another bad comment
            # yet another bad comment
        String foo = "bar"       # too much space for an inline comment
    }

    output {  # a fine comment
              # what about this one?
    
        # an OK comment
        String bar = foo

        Int a = 5 / 
            # a comment
            10
    }
}
