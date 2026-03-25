# ADR 009 (v2): The Multi-Task Binary & Provider Model

## Status
Accepted

## Context
In a standard Functions-as-a-Service (FaaS) or subprocess execution model, each task is compiled into a separate binary. For a workflow with 10 steps, a developer would have to manage 10 different Cargo projects, 10 binaries, and somehow communicate to the engine how they wire together. This creates massive overhead and fractures the development experience.

Furthermore, the Engine needs a way to discover the workflow graph (DAG/FSM) without relying on decoupled JSON files that can drift out of sync with the actual code.

## Decision
We adopt a "Terraform Provider" style execution model. An entire workflow (including all its tasks and its graph topology) is compiled into a **single Rust binary**. 

### The CLI Contract
The Engine interacts with this unified binary using a strict CLI argument contract:

1. **Discovery Phase (`--graph`)**
   On startup (or hot-reload), the Engine executes the binary:
   `./data/workflows/checkout_flow --graph`
   The binary's SDK intercepts this flag, bypasses all business logic, serializes the DAG topology to JSON, prints it to `stdout`, and exits cleanly. The Engine uses this output to build its internal registry and serve the UI.

2. **Execution Phase (`--execute-node <name>`)**
   When the Engine needs to run a specific step, it executes the exact same binary:
   `./data/workflows/checkout_flow --execute-node charge_stripe < input.json`
   The SDK intercepts this flag, routes the JSON payload from `stdin` to the registered `charge_stripe` function, executes it, and prints the result to `stdout`.

## Consequences
- **Positive:** Dramatic reduction in developer overhead. One workflow = one `Cargo.toml` = one binary. Shared structs and helper functions live in the same crate.
- **Positive:** Complete elimination of configuration drift. The code that executes the tasks is the exact same code that emits the DAG topology.
- **Positive:** Performance optimization. Because the Engine repeatedly spawns the exact same binary for every step in the workflow, the host Operating System keeps the binary hot in the memory page cache, significantly reducing process spawn latency.
- **Negative:** The binary size is slightly larger as it contains the code for all steps, though Rust's dead-code elimination and shared dependencies mitigate this.