<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [Intro](#intro)
    - [Chapter goals](#chapter-goals)
    - [Pre-requisites](#pre-requisites)
    - [Three flavours of concurrency](#three-flavours-of-concurrency)
    - [Type representation](#type-representation)
    - [Further reading](#further-reading)
- [The trouble with concurrency](#the-trouble-with-concurrency)
    - [Correctness](#correctness)
        - [Possible interleaving of events](#possible-interleaving-of-events)
    - [Performance](#performance)
        - [Mutual exclusion](#mutual-exclusion)
        - [Amdahl's law](#amdahls-law)
        - [Shared resource exhaustion](#shared-resource-exhaustion)
        - [False sharing](#false-sharing)
    - [The cost of scalability](#the-cost-of-scalability)
- [Concurrency models](#concurrency-models)
    - [Shared memory](#shared-memory)
    - [Worker pools](#worker-pools)
        - [Connection pools](#connection-pools)
    - [Actors](#actors)
- [Asynchrony and parallelism](#asynchrony-and-parallelism)
    - [Synchronisation primitives](#synchronisation-primitives)
- [Low-level concurrency](#low-level-concurrency)
    - [Memory operations](#memory-operations)
    - [Atomic types](#atomic-types)
    - [Memory ordering](#memory-ordering)
        - [Atomic memory order](#atomic-memory-order)
    - [Compare and exchange](#compare-and-exchange)
    - [The Fetch methods](#the-fetch-methods)
- [Sane concurrency](#sane-concurrency)
    - [Start simple](#start-simple)
    - [Write stress tests](#write-stress-tests)
    - [Use concurrency testing tools](#use-concurrency-testing-tools)
        - [Model checking with Loom](#model-checking-with-loom)
        - [Runtime checking with ThreadSanitizer](#runtime-checking-with-threadsanitizer)
    - [Heisenbugs](#heisenbugs)
- [Summary](#summary)

<!-- markdown-toc end -->

# Intro

## Chapter goals

**Do's:**

- How to add concurrency in Rust programs and libraries
- How to use concurrency primitives correctly

**Dont's:**

- How to implement concurrent data structures
- How to write high-performant code


## Pre-requisites

- experience with concurrent Rust
- basic familiarity with multi-core processor architectures
- basic understanding of concurrency


## Three flavours of concurrency

1. Single core, single thread: `async/await` - Ch. 8
2. Single core, many threads
3. Many cores (many threads): *true parallelism*.

More flavours, taking OS scheduling and pre-emption into account (*):

- Cooperative multi-tasking
- Pre-emptive multi-tasking

**Terminology**

- Concurrent - acting together while agreeing on something
- Parallel - running without intersections beside one another


## Type representation

- Only one aspect of concurrency: multi-threading
- Thread safety by type checking of `Send` and `Sync` contracts
- Not a compiler feature but a standard library *option*


## Further reading

- Fearless Concurrency in *The Rust Programming Language*:
  - `Send`, `Sync`, locks, smart pointers, channels.
- *Programming Rust* by Blandy, Orendorff and Tindall:
  - hands-on code examples


# The trouble with concurrency

## Correctness

**Difficulty #1:** coordinating access to resources

- Concurrent reads are in general simple
- Writes on the other hand can easily lead to *data races*, or *race conditions* more broadly

| Thread 1      | Thread 2      |
|:-------------:|:-------------:|
| x += 1        | x += 1        |

What is the value of x?
Every instruction consists of read, update and write back steps.

### Possible interleaving of events

| Thread 1      | Thread 2      |
|:-------------:|:-------------:|
| read x        |               |
|               | read x        |
| increment     |               |
|               | increment     |
| write x       |               |
|               | write x       |


## Performance

**Linear scalability** ideal: "the performance of the program scales with the number of cores"

In reality, scaling is sublinear... or worse, negative.

**Resource contention**: multiple threads attempt to access a shared resource concurrently.

Resources:

 - CPU cores
 - memory and cache lines
 - IO devices
 - locks and synchronisation primitives
 - drives and network bandwidth

 Consequences of contention:

 - increased latency
 - reduced throughput
 - potential deadlock

Remember simple combinatorics:

**Pigeonhole principle**: If n pigeons fly into m pigeonholes and n > m then at least one pigeonhole
must contain more than one pigeon.


### Mutual exclusion

Mutual exclusion is one of the main solutions to race conditions.

It can also be a cause for contention!

Key concepts:

 - *critical section*: a portion of code that accesses shared resources that can be accessed by at
   most one thread
 - *mutual exclusion*: at most one process can execute within a critical section at a time
 - *synchronisation mechanisms*: those that are used to enforce mutual exclusion, like locks,
   semaphores, monitors and mutexes

Examples:

 - OS or library functions that enforce single-threaded access to a critical section
   - Memory allocator in Rust use to require ME for some allocations
 - Resource dependencies between two parallel calls leading to sequential ordering in the kernel


### Amdahl's law

The speedup of a program from parallelisation is limited by the portion of the program that cannot
be parallelised.

S(N) = 1 / [(1 - P) + P / N]

where:

 - S(N) is the speedup when using N processors.
 - P is the fraction of the program that can be parallelised.
 - (1 - P) is the fraction of the program that is sequential (cannot be parallelised).
 - N is the number of processors.

Example:

TODO


### Shared resource exhaustion

 - parallel threads vastly exceeding the number of CPU or GPU cores
 - going out of memory
 - causing too many cache misses
 - exceeding maximum IO bandwidth: drives, network, PCIe, etc.

Fixes:

 - optimisation
 - new hardware


### False sharing

Two threads blocking on an entire resource while they use different parts of it.

Fixes:

 - splitting a lock in two, one for each part
 - removing locking of the resource in those threads
 - redesign, such as data structure padding, if the reason is outside the program, like in the
   example below:

**Cache line invalidation**, or **cache line ping-ponging**

Two processors write to different variables that reside on the same cache line. The coherence
protocol still invalidates and transfers the entire cache line, causing unnecessary contention. This
can be mitigated by padding data structures to avoid having frequently written variables share the
same cache line.

1. **Thread A Writes to Cache Line:**
 - If the cache line is in the **Shared** (S) state, the processor must obtain ownership.
 - The cache line transitions to **Modified** (M) or **Exclusive** (E) state.
 - Other processors' caches are invalidated, transitioning their cache lines to the **Invalid** (I)
   state.

2. **Thread B Writes to the Same Cache Line:**
 - If Thread B writes to the same cache line, and it is in Thread A's cache in the **Modified** (M)
   state, an invalidation message is sent to Thread A's cache.
 - Thread A's cache line transitions to the **Invalid** (I) state.
 - Thread B's cache line transitions to the **Modified** (M) or **Exclusive** (E) state after
   obtaining ownership.


## The cost of scalability

 - [Paper](https://www.frankmcsherry.org/assets/COST.pdf)


# Concurrency models

Rust has three concurrency models:

 - shared memory concurrency
 - worker pools
 - actors


## Shared memory

 - state guarded by a mutex
 - state stored in a hash map supporting concurrency, like `RwLock<HashMap>` or `DashMap`
 - can have data- or task-parallelism
 - fits usecases where shared state updates don't commute:
   - if f and g are state update functions, f(g(x)) != g(f(x))
 - locking comes with trade-offs that must be evaluated
   - for example, more concurrent reads but slower writes


### Example: state guarded by a mutex

```rust
fn shared_mem_mutex() -> usize {
    let count = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for _ in 0..10 {
        let count = Arc::clone(&count);
        let handle = thread::spawn(move || {
            let mut num = count.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result: MutexGuard<usize> = count.lock().unwrap();
    *result // Deref implementation gets the lock's data
}
```

### Example: state stored in a concurrent hash map

```rust
use dashmap::DashMap;

fn shared_mem_dashmap() -> usize {
    let count = Arc::new(DashMap::new());
    count.insert("value", 0);

    let mut handles = vec![];

    for _ in 0..10 {
        let count = Arc::clone(&count);
        let handle = thread::spawn(move || {
            let mut value = count.get_mut("value").unwrap();
            *value += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = count.get("value").unwrap();
    *result
}
```


## Worker pools

 - many identical threads receive jobs from a shared job queue
 - jobs are executed independently
 - shared memory is used for the job queue and result collection
 - work stealing - an idle thread can take other thread's job if that hasn't started yet
 - fits SIMD (single instruction, multiple data) applications


### Example: parallel matrix multiplication

```rust
use rayon::prelude::*;

fn worker_pool_matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let m = b[0].len();
    let p = b.len();

    assert!(
        a[0].len() == p,
        "Matrix dimensions do not match for multiplication!"
    );

    (0..n)
        .into_par_iter()
        .map(|i| {
            (0..m)
                .map(|j| (0..p).map(|k| a[i][k] * b[k][j]).sum())
                .collect()
        })
        .collect()
}
```

### Connection pools

 - a set of established connections that are provided on demand to threads that need connection
 - connection lifecycle management is a complex issue
   - every thread needs to be given a connection with no state from previous users

## Actors

### `riker`


# Asynchrony and parallelism

## Synchronisation primitives


# Low-level concurrency

## Memory operations

## Atomic types

## Memory ordering

### Atomic memory order

 - [C++20 reference](https://en.cppreference.com/w/cpp/atomic/memory_order)

## Compare and exchange

## The Fetch methods


# Sane concurrency

## Start simple

## Write stress tests

## Use concurrency testing tools

### Model checking with Loom

### Runtime checking with ThreadSanitizer

## Heisenbugs


# Summary
