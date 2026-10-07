/// The local-scope refusal fixture (`preserve-local-scope-across-exception-regions` 2.1/2.2): one
/// member per refusal family the change names, plus the two members the boundary is stated against.
///
/// Every member here is a Java 8 method with no debug tables; the names the recovery layer writes
/// are derived `localN` names, so every answer is a statement about control flow. The refusal
/// families:
///
/// * **protected-region fallback** — `savedAcrossFinally`: the saved local is read by the two
///   `finally` copies the walk cannot claim as one `finally`, so its definition–use slice crosses a
///   region that stays quoted;
/// * **protected-region fallback, computed handler join** — `handlerComputed`: the handler's value
///   is a conditional expression whose join lies outside the clause body the walk claims;
/// * **incomplete exception edge** — `nestedHandler`: nested rows reuse one handler slot and the
///   inner write's value is computed, so SSA cannot prove every path to the read reaches a
///   presented write;
/// * the boundary's two members: `siblingKept`, which has no crossing local and presents whole,
///   and `quotedSliceKept`, whose normal-path statements are written while the handler copy the
///   walk cannot claim stays a quoted block.
public final class ScopeRefusals {
    private ScopeRefusals() {}

    /// The saved local is written before the `try` and read by both `finally` copies.
    static int savedAcrossFinally(int n) {
        int saved = n;
        n = n + 1;
        try {
            n = n / (n - 2);
        } finally {
            n = saved;
        }
        return n;
    }

    /// The handler computes its value from a conditional whose join the clause walk does not claim.
    static int handlerComputed(int n) {
        int result;
        try {
            result = 100 / n;
        } catch (ArithmeticException e) {
            result = e.getMessage() == null ? -1 : -2;
        }
        return result;
    }

    /// Nested rows reuse one handler slot, and the inner write's value is a computed one.
    static int nestedHandler(int n) {
        int result = 0;
        try {
            try {
                result = 10 / n;
            } catch (ArithmeticException e) {
                result = -1;
            }
        } catch (RuntimeException e) {
            result = -2;
        }
        return result;
    }

    /// The independent sibling: no local of this member crosses a quoted region.
    static int siblingKept(int n) {
        int kept = n;
        try {
            kept = kept + 1;
        } catch (RuntimeException e) {
            kept = -1;
        }
        return kept;
    }

    /// The boundary inside one member: the normal path's statements are written and the handler
    /// copy the walk cannot claim stays a quoted block.
    static int quotedSliceKept(int n) {
        int outer = n;
        if (n > 0) {
            int saved = n;
            try {
                outer = outer + saved;
            } finally {
                saved = saved - 1;
            }
        }
        return outer;
    }

    static void log(int v) {
    }
}
