/// The nested/shared-handler escape fixture (`preserve-local-scope-across-exception-regions`
/// 2.1): the shared handler's two rows reach one handler entry, and the caught reference is bound
/// in the clause's own slot.
///
/// The committed `v8/ScopeRefusalsEscape.class` is this source; `escaped/ScopeRefusalsEscape.class`
/// is the documented control `patch-escape.py` derives from it, where the handler's entry store and
/// its load are rewritten to the method local's slot. That class is verifier-valid Java bytecode
/// whose caught value is read after the clause — a binding no Java lexical scope can spell, so the
/// recovery layer must refuse it instead of lifting the catch parameter into the method block.
public final class ScopeRefusalsEscape {
    private ScopeRefusalsEscape() {}

    static Object sharedHandler(Runnable action) {
        Object x = action;
        try {
            action.run();
        } catch (IllegalArgumentException | IllegalStateException e) {
            x = e;
        }
        return x;
    }
}
