#@ except: Todo

##This preamble comment is missing a space.
## 
## TODO: Line1 creates two diagnostics, one for CommentWhitespace and
## one for PreambleFormatting. Only one of the errors is expected.
##

version 1.1

workflow test {
    #@ except: DescriptionMissing
    meta {}

    output {}
}
