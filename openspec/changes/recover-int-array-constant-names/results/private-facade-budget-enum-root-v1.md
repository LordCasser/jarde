# Private facade budget-enum correction

Patch: `private-facade-budget-enum-root-v1.patch`  
Status: prepared for root review; not applied to product files.

The patch changes only the `BudgetDimension::` references inside `project_class_source_integer_constant_names` to the already-imported `CountedBudgetDimension::`. It adds no import and touches no other facade function.

This addresses the undeclared-type compiler errors reported in `results/scoped-rust-root-v2/2.stderr.raw`. No Git mutation or toolchain/test command was run for this patch.
