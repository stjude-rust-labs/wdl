#@ except: MetaSections
## This is a test for checking Object to struct coercion.

version 1.2

workflow test {
    #@ except: MetaDescription
    meta {}

    Struct myStruct
    myStruct = Object { String x}
    output {}
}
