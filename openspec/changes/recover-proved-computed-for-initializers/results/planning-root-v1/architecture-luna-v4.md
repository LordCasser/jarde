# CF07 computed initializer: can the existing Builder carry it?

Static-only supplement to `computed-init-for-next-design-luna-v3.md`; no code/spec change or runtime result.

## Finding

A hard-coded `Load(arg); Push(1); Sub` template is not justified by the architecture. `for_header_candidate` needs to certify the induction-slot/control-flow facts and that the initializer really is the unique preheader store value. It need not reconstruct or whitelist the Java expression: Builder already owns expression reconstruction and conservative refusal.

Keep the existing proof unchanged except the initializer gate:

- Preheader remains the unique outside predecessor and its only successor is this header; its final SSA instruction is still `Store(slot)`.
- The exact stack value read by that store is defined by an `Instruction` in this same preheader block; the store writes `slot`. Keep the current exact header phi inputs and one-use `init_value`/`update_value` checks. Also bind the stack input to this store's actual read (one instruction use at this store), rather than accepting a matching BCI alone.
- Retain update shape, induction-input/phi identity, test/invariant checks, all in-loop writes checks, and rejection of induction-slot reads/writes outside loop + preheader (except the separately certified return-leaf read proposed in v3). Do not add opcodes or alter CFG ownership.

## Why no initializer purity walker is indicated

`Builder::instruction` turns the terminal `Store` into the existing `Declare { value: Some(..) }` or `Assign` at that store BCI, using `render_value` on its exact stack operand. That renderer already recursively reconstructs supported arithmetic, conversions, calls, fields, arrays, etc.; unsupported or untyped expressions refuse instead of producing a For header. No new expression builder is needed to turn the resulting statement into the `For` initializer.

`for_init_index` selects that exact statement by `proof.init_bci`, requires it to be a declaration-with-value or assignment, and requires every later statement before the loop to be an empty declaration. It removes only that statement and embeds it as `StmtKind::For.init`; nonempty statements stay in place. Therefore the initializer still executes once, at the same statement position before the first condition. A declaration's scope narrows to the `for`; the current slot-access scan already rejects induction-slot uses after the loop. If the init statement and loop are in different Builder statement contexts (for example, split by a Guard), the lookup fails and Builder refuses the For projection.

Evaluation effects are not a reason for an `arg-1`-only rule. `prepare_deferred_bindings` already examines expression producers, one-use consumers, and independent instructions between producer and reader. For supported effectful producers (`Invoke`, claimed field/array reads, division/remainder, casts, construction sites), it materializes a binding at the producer when an independent boundary would otherwise reorder evaluation. `render_value` preserves operand order for its supported expression forms; Builder falls back when it cannot render a value. The preheader's ordinary statements are still walked in bytecode order. This is the existing mechanism that must be relied on, rather than reimplementing effect classification in the region layer.

## Counterexample to only checking the three root facts

A verifier-valid preheader can leave an int result on the operand stack, execute an independent effect (`invokestatic f()I; invokestatic g()V`), then store the pending result. If Builder simply deferred `f()` into the store/For initializer, it would print `g(); for (... = f(); ...)`, reversing call/exception order. The current Builder has the deferred-binding/independent-boundary path specifically for this shape; accept it only if the actual producer is bound at its original point or the renderer refuses. The root facts alone do not establish that. Likewise a duplicated/shared effectful SSA value must not be textually evaluated twice; existing consumer-count/refusal behavior is relevant, but its exact interaction with the new For path should be confirmed on real IR.

The same-position argument also assumes the chosen init `Stmt` is in the same lexical/exception context as the loop. `for_init_index` uses the current statement vector, which is a strong local boundary; nevertheless confirm Guard/handler context for the candidate rather than inferring it from the linear BCI sequence. Do not permit moving an init across a nonempty statement, a region/context boundary, or a handler boundary.

## Minimal diagnostic / remaining unknowns

Use the existing frozen-class real-IR setup to record: `stack_input` definition block/BCI and exact uses; the full dependency DAG and shared values; the interval from effectful producers to BCI 3 and any `prepare_deferred_bindings` plan; whether the built BCI-3 statement is the one selected by `for_init_index`; and whether it shares the loop's Builder/handler context. Test both a no-intervening-effect chain and an independent-effect chain if present in a real fixture. No new purity table is warranted if the existing Builder either materializes at the original point or refuses.

Still unproved statically: that this precise frozen CF07 operand passes `render_value` and the materialization checks; that the init declaration has no Java-name/scope collision in the generated For; and that moving only the statement container preserves all exception-table regions for this real method. Confirm through actual class-source, source-map, compile, baseline and full-class checks after the other two v3 gates are separately proven. The old `ForBoundaries` refusal cases (`observedAfter`, `twoUpdates`, `varyingBound`, `noUpdate`) and budget/cancellation tests remain required controls.
