#@ except: DescriptionMissing, MatchingParameterMeta

version 1.2

task foo {
    meta {
        description: {
            help: "test"
        }
        help: {
            name: "something",
            other: "another"
        }
        foo: {
            bar: "baz",
            baz: "quux" ,
        }
    }

    parameter_meta {
        bam: "Input BAM format file to generate coverage for"
        gtf: "Input genomic features in gzipped GTF format to count reads for"
        strandedness: {
            description: "Strandedness protocol of the RNA-Seq experiment",
            external_help: "https://htseq.readthedocs.io/en/latest/htseqcount.html#cmdoption-htseq-count-s",
            choices: [
                "yes",
                "reverse",
                "no"
            ]
        }
        minaqual: {
            description: "Skip all reads with alignment quality lower than the given minimum value",
            common: true
        }
        modify_memory_gb: "Add to or subtract from dynamic memory allocation. Default memory is determined by the size of the inputs. Specified in GB."
        modify_disk_size_gb: "Add to or subtract from dynamic disk space allocation. Default disk size is determined by the size of the inputs. Specified in GB."
        not_an_option: {
            name: "test"
        }
   }

   input {}

   command <<< >>>

   output {}

   runtime {}

}
