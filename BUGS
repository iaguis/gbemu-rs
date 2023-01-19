* Fix sprite appearing on the other side of the screen bug
* Fix tearing (double buffering?)
* Fix Mario crash in World 2-1
    ```
    thread 'main' panicked at 'index out of bounds: the len is 23040 but the index is 18446744073709551614', src/gpu.rs:558:29
    ```
    * Might be related to lack of 8x16 sprite support
    * The index is 2^64-2 ((2^64 - 1) - 1)
        * Is it i16 with sign extended?
