public class InvocationAuditRunner {
    public static void main(String[] ignored) {
        System.out.println("super=" + new InvocationAudit(0).code);
        System.out.println("this=" + new InvocationAudit('x').code);
        System.out.println("objectConstructor=" + new InvocationAudit((Object) "x").code);
        System.out.println("stringConstructor=" + new InvocationAudit("x").code);
        AuditEffects.count = 0;
        System.out.println("ordered=" + InvocationAudit.ordered() + ":" + AuditEffects.count);
        AuditEffects.count = 0;
        try { InvocationAudit.failedFirst(); } catch (IllegalArgumentException error) {
            System.out.println("first=" + AuditEffects.count);
        }
        AuditEffects.count = 0;
        try { InvocationAudit.failedSecond(); } catch (IllegalArgumentException error) {
            System.out.println("second=" + AuditEffects.count);
        }
        System.out.println("objectIdentity=" + InvocationAudit.objectIdentity("x"));
        System.out.println("stringIdentity=" + InvocationAudit.stringIdentity("x"));
        System.out.println("castIdentity=" + InvocationAudit.castIdentity("x"));
        System.out.println("castNull=" + InvocationAudit.castIdentity(null));
        try { InvocationAudit.castIdentity(new Object()); } catch (ClassCastException error) {
            System.out.println("castBad=CCE");
        }
    }
}
