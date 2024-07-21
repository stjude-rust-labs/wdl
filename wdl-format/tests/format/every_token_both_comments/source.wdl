#1a
version #1b
#2a
1.0 #2b
#3a
workflow #3b
#4a
test_wf #4b
#5a
{
input
{
SpliceJunctionMotifs
 out_sj_filter_overhang_min
  = 
   SpliceJunctionMotifs
    {
noncanonical_motifs
:
 30
 , 
GT_AG_and_CT_AC_motif 
: 
12 
, 
} 
}
parameter_meta
{
out_sj_filter_overhang_min: {
type: "SpliceJunctionMotifs",
label: "Minimum overhang required to support a splicing junction"
}
}
output
{
SpliceJunctionMotifs KAZAM = out_sj_filter_overhang_min
}
#6a
meta #6b
#7a
{ #7b
#8a
description #8b
#9a
: #9b
#10a
 "Test workflow" #10b
#11a
} #11b

call test_task as foo {
input: bowchicka = "wowwow"
}
if (
true
) {

call test_task after foo {
input: bowchicka = "bowchicka"
}
scatter (i in range(3)) {
call test_task as bar {
input: bowchicka = i * 42
}
}
}

}
task
test_task
{
command <<<>>>
input {
String bowchicka
}
parameter_meta {
bowchicka: {
type: "String",
label: "Bowchicka"
}
}
}

struct SpliceJunctionMotifs {
Int noncanonical_motifs
Int GT_AG_and_CT_AC_motif
}