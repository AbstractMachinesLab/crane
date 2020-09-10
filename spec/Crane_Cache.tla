How does the caching mechanism work?

For every node in the build graph there'll be certain inputs

Those inputs should be content-hashed, and the cache should keep a map of
the hashes and their last value, and of the their paths to the last hash.

## Input Caching

If any of those hashes is missing in the cache, then the input has changed.
We can then create a new cache entry with the new contents, and point the path
to the new hash.

If none of the inputs have changed, then we do nothing.

## Build Node Caching

If any input hash is new or has changed, a new hash for the entire build node
will be created. This hash should include all of the inputs used for this build node:
- all inputs
- toolchain
- all build node configuration
- all dependent build node hashes

This hash will then be used to save the output of the build node after it is built.

This will ensure that if any of the inputs to this output are changed, we rebuild it.

## Ouput Caching

After building, the hash of the inputs will be used to create a cache entry with
the output contents in it.

Additionally, the output path will be added to a map pointing to the output hash.


# Flow: BuildGraph Cache Hits/Miss

For any given target build node, we should be able to hash its inputs to find its output.

In the process of hashing its inputs, we will traverse the build graph going upwards,
recursively repeating the same process until we reach nodes with no dependencies.

Then we traverse back down to the target node, computing the relevant build node hashes
and using them to find the corresponding outputs in the cache.

If we find a node without output hashes, we can mark the rest of the tree as dirty.

This dirty tree needs rebuilding and can rely on the cached outputs of all of its dependencies.


# Flow: Cache Clean

Marking the entire cache as removable, we can traverse the entire build graph, hashing
inputs and nodes, and checking if they exist in the cache.

Any hash that is a cache hit should be marked as not removable.

After the traversal is done, all cache entries still marked as removable can be safely
removed.

---------------------------- MODULE Crane_Cache ----------------------------


=============================================================================
\* Modification History
\* Last modified Wed Sep 09 12:07:55 CEST 2020 by ostera
\* Created Wed Sep 09 12:07:17 CEST 2020 by ostera