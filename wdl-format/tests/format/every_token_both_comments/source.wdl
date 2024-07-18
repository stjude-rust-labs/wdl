#1
version #1
#2
1.0#2
#3
workflow #3
#4
test_wf #4
#5
{ #5
#6
input #6
{ #7
SpliceJunctionMotifs #8
 out_sj_filter_overhang_min #9
  = #10
   SpliceJunctionMotifs #11
    { #12
noncanonical_motifs #13
: #14
 30 #15
 , #16
GT_AG_and_CT_AC_motif #17
: #18
12 #19
, #20
} #21
}#22
parameter_meta#23
{ #24
out_sj_filter_overhang_min #25
: #26
{ #27
type #28
: #29
 "SpliceJunctionMotifs" #30
 , #31
label #32
: #33
 "Minimum overhang required to support a splicing junction" #34
} #35
} #36
output #37
{ #38
SpliceJunctionMotifs #39
KAZAM #40
 = #41
  out_sj_filter_overhang_min #42
} #43
meta #44
{ #45
description #46
: #47
 "Test workflow" #48
} #49

call#50
 test_task #51
  as #52
   foo  #53
   { #54
input #55
: #56
 bowchicka #57
  = #58
   "wowwow"#59
} #60
if #61
 ( #62
true #63
) #64
{ #65

call #66
 test_task #67
  after #68
   foo #69
    { #70
input #71
: #72
 bowchicka #73
  = #74
   "bowchicka" #75
} #76
scatter #77
 ( #78
 i #79
  in #80
   range # 81
   ( #82
   3 #83
   ) #84
   ) #85
   { #86
call #87
test_task #88
 as #89
  bar #90
   { #91
input #92
: #93
 bowchicka #94
  = #95
   i #96
    * #97
     42 #98
} #99
} #100
} #101

} #102
task #103
test_task #104
{ #105
command #106
 <<< #107
 >>> #108
input #109
{ #110
String #111
 bowchicka #112
} #113
parameter_meta #114
{ #115
bowchicka #116
: #117
 { #118
type #119
: #120
 "String" #121
 , #122
label #123
: #124
 "Bowchicka" #125
} #126
} #127
} #128

struct #129
 SpliceJunctionMotifs #130
  { #131
Int #132
 noncanonical_motifs #133
Int GT_AG_and_CT_AC_motif #134
} #135
