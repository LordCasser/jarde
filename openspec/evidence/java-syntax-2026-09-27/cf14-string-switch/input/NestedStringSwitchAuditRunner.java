public class NestedStringSwitchAuditRunner {
    public static void main(String[] args) {
        for (String value : new String[] {"a", "b", "c", "d", null}) {
            try {
                System.out.println(value + "=" + NestedStringSwitchAudit.choose(value));
            } catch (RuntimeException error) {
                System.out.println(value + "=throws=" + error.getClass().getSimpleName());
            }
        }
    }
}
