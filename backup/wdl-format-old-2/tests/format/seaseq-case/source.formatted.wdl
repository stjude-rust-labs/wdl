version 1.0

import "workflows/tasks/bedtools.wdl"
import "workflows/tasks/bowtie.wdl"
import "workflows/tasks/fastqc.wdl"
import "workflows/tasks/macs.wdl"
import "workflows/tasks/rose.wdl"
import "workflows/tasks/runspp.wdl"
import "workflows/tasks/samtools.wdl"
import "workflows/tasks/seaseq_util.wdl" as util
import "workflows/tasks/sicer.wdl"
import "workflows/tasks/sortbed.wdl"
import "workflows/tasks/sratoolkit.wdl" as sra
import "workflows/workflows/bamtogff.wdl"
import "workflows/workflows/mapping.wdl"
import "workflows/workflows/motifs.wdl"
import "workflows/workflows/visualization.wdl" as viz
