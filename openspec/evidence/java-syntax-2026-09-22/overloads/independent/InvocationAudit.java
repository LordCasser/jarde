public class InvocationAudit extends PrologueParent {
    public InvocationAudit(Object value) { super(value); }
    public InvocationAudit(String value) { super(value); }
    public InvocationAudit(int ignored) { super((Object) "x"); }
    public InvocationAudit(char ignored) { this((Object) "x"); }
    public static String mark(int digit, boolean fail) {
        return AuditEffects.mark(digit, fail);
    }
    public static int select(Object first, Object second) { return 41; }
    public static int select(String first, String second) { return 42; }
    public static int ordered() { return select((Object) mark(1, false), (Object) mark(2, false)); }
    public static int failedFirst() { return select((Object) mark(1, true), (Object) mark(2, false)); }
    public static int failedSecond() { return select((Object) mark(1, false), (Object) mark(2, true)); }
    public static int objectIdentity(Object value) { return select(value, value); }
    public static int stringIdentity(String value) { return select(value, value); }
    public static int castIdentity(Object value) { return select((String) value, (String) value); }
}
