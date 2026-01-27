```text
   ██████╗  ██████╗  ██████╗ ██╗████████╗ █████╗ ████████╗ ██████╗ ██████╗
  ██╔════╝ ██╔═══██╗██╔════╝ ██║╚══██╔══╝██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
  ██║      ██║   ██║██║  ███╗██║   ██║   ███████║   ██║   ██║   ██║██████╔╝
  ██║      ██║   ██║██║   ██║██║   ██║   ██╔══██║   ██║   ██║   ██║██╔══██╗
  ╚██████╗ ╚██████╔╝╚██████╔╝██║   ██║   ██║  ██║   ██║   ╚██████╔╝██║  ██║
   ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝



Cogitator is a deterministic execution and witnessed-telemetry framework for scientifically auditable evaluation of autonomous cyber defence systems.

Modern agentic security systems are increasingly powerful, but their evaluation remains fragile: stochastic inference, tool-mediated nondeterminism, parallel execution drift, and mutable logging make results difficult to reproduce or verify. Cogitator treats an evaluation run as a deterministic program whose full event trace is cryptographically committed, enabling rebuildable, replayable, third-party validation.

---

## Core idea

Cogitator enforces a simple scientific invariant:

Same environment + same input + same seed  
→ same trajectory  
→ same witness root

Rather than trusting narrative logs, Cogitator produces tamper-evident witness chains where every execution event contributes to a cryptographic commitment. Any insertion, deletion, mutation, or reordering changes the final witness root.

---

## Key contributions

Cogitator provides:

- Deterministic execution kernel for agentic cyber evaluation
- Cryptographic witness chains committing to full causal traces
- Explicit entropy budgeting to make randomness measurable
- Reproducible evaluation environments grounded in NixOS derivations
- Standalone verification of execution traces via witness recomputation

---

## Witness chains

Cogitator commits to the entire event trace using a sequential hash chain:

h_0     = BLAKE3(“COGITATOR” || metadata)
h_{t+1} = BLAKE3(h_t || encode(event_t))

The final value, `witness_root = h_T`, uniquely commits to the run’s execution history.

This enables post-hoc verification:

- If the witness root matches, the trace is authentic
- If any event is altered, the commitment breaks

---

## Entropy budgeting

Agent evaluations often hide randomness behind sampling temperatures, planner branches, tool timeouts, or scheduler jitter.

Cogitator treats randomness as an explicit audited resource:

- entropy sources are declared in metadata
- consumption is recorded in the trace
- evaluations become comparable across models and runs

Randomness becomes measurable rather than implicit.

---

## Reproducibility via NixOS

Cogitator is designed to run inside reproducible NixOS environments.

This enables:

- pinned dependency graphs
- hermetic toolchains
- rebuildable experiments
- bit-identical evaluation pipelines
- third-party verifiable re-execution from the same derivation

In practice, published results can be reproduced from a flake lock and verified by recomputing the witness root.

---

## Artifact bundle (planned)

Cogitator will release an artifact package containing:

- Nix flake pinning all dependencies and runtimes
- Deterministic execution kernel and tool wrappers
- Canonical trace schema specification
- Standalone verifier for witness root recomputation
- Regression suite demonstrating stable witness roots across runs

---

## Threat model

Cogitator addresses:

- accidental nondeterminism (parallelism, scheduling drift)
- post-hoc log editing or trace fabrication

Cogitator does not attempt to defend against:

- fully malicious host substrates (compromised hypervisors or OS)

The framework targets scientific auditability under declared pinned environments.

---

## Status

Cogitator is an active research project. The current repository is intended as the reference implementation accompanying the Cogitator paper and artifact release.

---

## References

- Guo et al. R2: Record and Replay at the Application Level. OSDI 2008  
- Malka et al. Functional Package Management Enables Reproducible Builds at Scale. arXiv 2025  
- Aumasson et al. The BLAKE3 Hashing Framework. IETF draft 2024  
- Trillian: Merkle-tree-backed verifiable logs. transparency.dev  

---

## License

To be released under an OSI-approved open source license.