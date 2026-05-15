# spill-to-disk

## Summary

Core spill-to-disk infrastructure for memory-bounded operators.

## Functional Requirements

1. `AdaptiveMemoryTracker` must track allocation/deallocation with CAS loop
2. `PartitionManager` must create/read/cleanup partition files
3. Memory threshold at 70% triggers soft spill signal
4. Memory limit at 90% triggers hard spill requirement

## Technical Constraints

- Use `std::sync::atomic` for lock-free tracking
- Use `bincode` for serialization
- Temp files in `std::env::temp_dir()` by default
