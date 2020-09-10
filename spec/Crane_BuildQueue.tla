How can we ensure that we build everything as concurrently as possible, correctly?

All of our build graphs should be DAGs. So no cycles whatsoever.

When the build starts, we ask it to build a specific target node.

Starting from that node, we should traverse the graph "upwards", finding its
dependencies, until we reach the edge nodes that have no dependencies.

If the queue is built following the dependency graph that leads to the target node,
that is, first in queue are the outer-most nodes in the dependency graph, and last
in the queue is the target node itself, with no node in the queue depending
on nodes that come after it, then this would be a safe way of parallely executing the
work queue.

The build graph:

A -> B -> C -> F
  \-> D -> E  /

would yield any of the queues:

F E C D B A
F E C B D A
F E D C B A
F C B E D A
F C E B D A
F C E D B A

since they all would result in the same dependency resolution.

-------------------------- MODULE Crane_BuildQueue --------------------------

=============================================================================
\* Modification History
\* Last modified Wed Sep 09 12:11:32 CEST 2020 by ostera
\* Created Wed Sep 09 12:10:05 CEST 2020 by ostera
