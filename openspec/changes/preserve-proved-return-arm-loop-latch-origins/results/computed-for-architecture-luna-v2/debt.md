# Deferred debt

The existing counted-loop proof requires a constant `Push(Int)` preheader initializer, then rejects every induction-slot access outside the natural loop and its preheader. CF07 `lastIndexOf` has a computed decrement initializer and a certified return leaf that reads the induction variable, so relaxing the initializer producer shape alone cannot cover it.

A future change should first prove whether the existing terminal-return certificate can be computed early and reused as a narrow allowance for out-of-loop reads. Keep writes rejected unless a separate semantic argument and real-IR evidence justify them. Independently assess `implicit_tail_latch_origin`: its direct return/latch-If path is currently disabled when `ForHeader` exists. Do not bundle either change into the frozen return-origin product or claim acceptance before full source-map and whole-class validation.
