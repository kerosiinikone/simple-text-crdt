# simple-text-crdt

While learning Rust; some implementation choices and paper logic translation done with the help of Gemini 2.5. Early on I decided on using an extensive amount of reference counted RefCells to store nodes and refer to them in the recursion. Later on this approach proved to be quite annoying to deal with.

### Some notes for myself

- All updates get replicated on each peer replica
- Full state-based sync or a delta-state sync?
- **Treedoc**, set CRDT
- Commutative replicated data types -> a linear sequence of characters as _atoms_
- Atoms have a PosID (with total order among others)
- PosIDs through BTs

### Resources:

- **Main Reference** https://inria.hal.science/inria-00445975/document
- https://arxiv.org/pdf/1805.06358
- https://mattweidner.com/2023/09/26/crdt-survey-1.html
- https://mattweidner.com/2023/09/26/crdt-survey-2.html
- https://mattweidner.com/2023/09/26/crdt-survey-3.html
- https://mattweidner.com/2023/09/26/crdt-survey-4.html
- https://github.com/pfrazee/crdt_notes/tree/68c5fe81ade109446a9f4c24e03290ec5493031f#portfolio-of-basic-crdts
- https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type

### More

- https://crdt.tech/resources

### TODOs

- Add a .vscode for formatting (shorter max line length)
