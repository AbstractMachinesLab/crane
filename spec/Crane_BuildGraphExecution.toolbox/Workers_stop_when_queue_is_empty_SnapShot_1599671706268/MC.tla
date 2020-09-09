---- MODULE MC ----
EXTENDS Crane_BuildGraphExecution, TLC

\* CONSTANT definitions @modelParameterConstants:0Workers
const_1599671700237193000 == 
2
----

\* CONSTANT definitions @modelParameterConstants:1Nodes
const_1599671700237194000 == 
5
----

\* Constant expression definition @modelExpressionEval
const_expr_1599671700237195000 == 
{ nodes[n]: n \in 1..Nodes }
----

\* Constant expression ASSUME statement @modelExpressionEval
ASSUME PrintT(<<"$!@$!@$!@$!@$!",const_expr_1599671700237195000>>)
----

=============================================================================
\* Modification History
\* Created Wed Sep 09 19:15:00 CEST 2020 by ostera
