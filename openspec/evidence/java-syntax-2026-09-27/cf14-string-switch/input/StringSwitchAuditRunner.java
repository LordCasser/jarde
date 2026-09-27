public class StringSwitchAuditRunner {
    public static void main(String[] args) {
        for (String value : new String[] {"frewhyh", "phgafkp", "ucguedt", "test", "test2", "other", "unknown", null}) {
            StringSwitchAudit.calls = 0;
            try {
                System.out.println(value + "=" + StringSwitchAudit.choose(value)
                        + ":calls=" + StringSwitchAudit.calls);
            } catch (RuntimeException error) {
                System.out.println(value + "=throws=" + error.getClass().getSimpleName()
                        + ":calls=" + StringSwitchAudit.calls);
            }
        }
    }
}
