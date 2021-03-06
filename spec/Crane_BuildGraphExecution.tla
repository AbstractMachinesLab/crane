This spec models the behavior of a build graph execution worker.

It assumes that there is a build queue, where nodes are consumed from the front to the back,
and have been inserted in topological sorting (so the first node to be consumed has no dependencies).
 
Each build graph node in the queue can be in one of the following states:
- "waiting", or waiting o be picked up by a build worker
- "building", meaning it is currently being built by a worker
- "built", meaning it was built correctly by a worker
- "cached", meaning the worker identified a cache-hit
- "errored", meaning the worker found a problem with this build node

This tiny state machine allows workers to pull work from a queue, and check if 
the dependencies of their current job are met or not. If they are, they will proceed
to build it, if they aren't, then they need to wait.

Actually building a node may be met with a Cache hit. This wasn't strictly necessary to model
since the result is a built node nonetheless.

In this spec we don't quite care how the build graph is built, so we'll explore
both the case when some node checks for dependencies and doesn't. This should
be enough  to figure out how the actual worker will behave with a real build graph.
Abstraction for the win. 

At the end of the execution, the queue should be emptied, and all nodes should be built or cached OR
the process should have been aborted with some errored nodes.


--------------------- MODULE Crane_BuildGraphExecution ---------------------
EXTENDS Naturals, Integers, Sequences

CONSTANT Nodes
CONSTANT Workers
ASSUME NodesInRange == Nodes \in Nat /\ Nodes >= 1
ASSUME WorkersInRange == Workers \in Nat /\ Workers >= 1


(*--algorithm build_graph_execution

variables
    abort = FALSE,
    queue = Nodes,
    nodes = [ n \in 1..Nodes |-> "waiting" ],
    work_done = {}
;

define
    TypeInvariant == queue \in Nat
    
    Statuses == { nodes[n]: n \in 1..Nodes }
    AllBuiltOrCached == Statuses \subseteq { "built", "cached" }
    SomeErrored == "errored"  \in Statuses
    
    EventuallyQueueIsConsumed == <>[]( abort \/ (~abort /\ queue = 0 ))
    NoWorkIsDoneTwice == <>[]( abort \/ (~abort /\ work_done = 1..Nodes ) )
    EitherWeAbortOrThereAreNoErrors == <>[](  (abort /\ SomeErrored)  \/  (~abort /\ AllBuiltOrCached) )
end define;

fair process Worker \in 1..Workers
variables
    current_job = -1
,
begin
    Loop: 
        if abort \/ queue = 0 then
            goto Done;
         else
         
        current_job := queue;
        nodes[current_job] := "building";
        queue := queue - 1;
        goto Work;
     end if;
    
    Work:
        when current_job /= -1; 
        
         either CheckOnDependencies:
               if current_job < Nodes /\ (nodes[current_job+1] = "built" \/ nodes[current_job+1] = "cached") then
                 goto BuildNode;
               else
                   WaitForDependency:
                    while ~abort /\ current_job < Nodes /\ nodes[current_job + 1] = "building" do
                        skip;
                    end while;
                    if ~abort then goto BuildNode;
                    else goto Done;
                    end  if;
                end if;
        or BuildNode:
            either   WorkSucceeds:
                nodes[current_job] := "built";
                work_done := work_done \union {current_job};
                goto Loop;
            or CacheHit:
                nodes[current_job] := "cached";
                work_done := work_done \union {current_job};
                goto Loop;
            or BuildError:
                 nodes[current_job] := "errored";
                abort := TRUE;
                goto Loop;
            end either;
    end either;
end process;

end algorithm; *)
\* BEGIN TRANSLATION - the hash of the PCal code: PCal-3054d4d62f832d2509536c57dc70d748
VARIABLES abort, queue, nodes, work_done, pc

(* define statement *)
TypeInvariant == queue \in Nat

Statuses == { nodes[n]: n \in 1..Nodes }
AllBuiltOrCached == Statuses \subseteq { "built", "cached" }
SomeErrored == "errored"  \in Statuses

EventuallyQueueIsConsumed == <>[]( abort \/ (~abort /\ queue = 0 ))
NoWorkIsDoneTwice == <>[]( abort \/ (~abort /\ work_done = 1..Nodes ) )
EitherWeAbortOrThereAreNoErrors == <>[](  (abort /\ SomeErrored)  \/  (~abort /\ AllBuiltOrCached) )

VARIABLE current_job

vars == << abort, queue, nodes, work_done, pc, current_job >>

ProcSet == (1..Workers)

Init == (* Global variables *)
        /\ abort = FALSE
        /\ queue = Nodes
        /\ nodes = [ n \in 1..Nodes |-> "waiting" ]
        /\ work_done = {}
        (* Process Worker *)
        /\ current_job = [self \in 1..Workers |-> -1]
        /\ pc = [self \in ProcSet |-> "Loop"]

Loop(self) == /\ pc[self] = "Loop"
              /\ IF abort \/ queue = 0
                    THEN /\ pc' = [pc EXCEPT ![self] = "Done"]
                         /\ UNCHANGED << queue, nodes, current_job >>
                    ELSE /\ current_job' = [current_job EXCEPT ![self] = queue]
                         /\ nodes' = [nodes EXCEPT ![current_job'[self]] = "building"]
                         /\ queue' = queue - 1
                         /\ pc' = [pc EXCEPT ![self] = "Work"]
              /\ UNCHANGED << abort, work_done >>

Work(self) == /\ pc[self] = "Work"
              /\ current_job[self] /= -1
              /\ \/ /\ pc' = [pc EXCEPT ![self] = "CheckOnDependencies"]
                 \/ /\ pc' = [pc EXCEPT ![self] = "BuildNode"]
              /\ UNCHANGED << abort, queue, nodes, work_done, current_job >>

CheckOnDependencies(self) == /\ pc[self] = "CheckOnDependencies"
                             /\ IF current_job[self] < Nodes /\ (nodes[current_job[self]+1] = "built" \/ nodes[current_job[self]+1] = "cached")
                                   THEN /\ pc' = [pc EXCEPT ![self] = "BuildNode"]
                                   ELSE /\ pc' = [pc EXCEPT ![self] = "WaitForDependency"]
                             /\ UNCHANGED << abort, queue, nodes, work_done, 
                                             current_job >>

WaitForDependency(self) == /\ pc[self] = "WaitForDependency"
                           /\ IF ~abort /\ current_job[self] < Nodes /\ nodes[current_job[self] + 1] = "building"
                                 THEN /\ TRUE
                                      /\ pc' = [pc EXCEPT ![self] = "WaitForDependency"]
                                 ELSE /\ IF ~abort
                                            THEN /\ pc' = [pc EXCEPT ![self] = "BuildNode"]
                                            ELSE /\ pc' = [pc EXCEPT ![self] = "Done"]
                           /\ UNCHANGED << abort, queue, nodes, work_done, 
                                           current_job >>

BuildNode(self) == /\ pc[self] = "BuildNode"
                   /\ \/ /\ pc' = [pc EXCEPT ![self] = "WorkSucceeds"]
                      \/ /\ pc' = [pc EXCEPT ![self] = "CacheHit"]
                      \/ /\ pc' = [pc EXCEPT ![self] = "BuildError"]
                   /\ UNCHANGED << abort, queue, nodes, work_done, current_job >>

WorkSucceeds(self) == /\ pc[self] = "WorkSucceeds"
                      /\ nodes' = [nodes EXCEPT ![current_job[self]] = "built"]
                      /\ work_done' = (work_done \union {current_job[self]})
                      /\ pc' = [pc EXCEPT ![self] = "Loop"]
                      /\ UNCHANGED << abort, queue, current_job >>

CacheHit(self) == /\ pc[self] = "CacheHit"
                  /\ nodes' = [nodes EXCEPT ![current_job[self]] = "cached"]
                  /\ work_done' = (work_done \union {current_job[self]})
                  /\ pc' = [pc EXCEPT ![self] = "Loop"]
                  /\ UNCHANGED << abort, queue, current_job >>

BuildError(self) == /\ pc[self] = "BuildError"
                    /\ nodes' = [nodes EXCEPT ![current_job[self]] = "errored"]
                    /\ abort' = TRUE
                    /\ pc' = [pc EXCEPT ![self] = "Loop"]
                    /\ UNCHANGED << queue, work_done, current_job >>

Worker(self) == Loop(self) \/ Work(self) \/ CheckOnDependencies(self)
                   \/ WaitForDependency(self) \/ BuildNode(self)
                   \/ WorkSucceeds(self) \/ CacheHit(self)
                   \/ BuildError(self)

(* Allow infinite stuttering to prevent deadlock on termination. *)
Terminating == /\ \A self \in ProcSet: pc[self] = "Done"
               /\ UNCHANGED vars

Next == (\E self \in 1..Workers: Worker(self))
           \/ Terminating

Spec == /\ Init /\ [][Next]_vars
        /\ \A self \in 1..Workers : WF_vars(Worker(self))

Termination == <>(\A self \in ProcSet: pc[self] = "Done")

\* END TRANSLATION - the hash of the generated TLA code (remove to silence divergence warnings): TLA-c70b83f43df290b37b53d98b93983f83

=============================================================================
\* Modification History
\* Last modified Wed Sep 09 19:38:13 CEST 2020 by ostera
\* Created Wed Sep 09 12:07:17 CEST 2020 by ostera
