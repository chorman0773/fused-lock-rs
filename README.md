# Fused Lock

This library provides a type, FusedRwLock, which has exclusive/shared semantics.

At any time, one thread may hold an exclusive lock over contents,
 or any number of threads may hold a shared lock to those contents. 
 However, unlike a standard RwLock, after being locked shared at any point,
  it becomes impossible to acquire an exclusive lock again. 

This may provide an advantage over using a standard RwLock, any time you have a value that's only written to once, and, after being read, is never written again (for example, a registry that's locked before it's accessed, or a resource loading scheme)

## License

Copyright (C) 2021 Connor Horman.

This software is dual-licensed under the terms of the MIT and Apache v2 license. See LICENSE-MIT and LICENSE-APACHE for details. 

Any contribution intentionally submitted by you for inclusion in this repository must be dual-licensed as above. 
