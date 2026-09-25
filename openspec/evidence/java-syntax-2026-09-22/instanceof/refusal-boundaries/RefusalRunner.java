import java.lang.reflect.InvocationTargetException;

/** Source-only runner for legal method_info refusal variants. */
public final class RefusalRunner {
    private RefusalRunner() {}

    public static void main(String[] args) {
        if (args.length != 1) {
            throw new IllegalArgumentException("mode required");
        }
        switch (args[0]) {
            case "pop":
                pop();
                break;
            case "duplicate":
                System.out.println("duplicate-true=" + InstanceOfProbe.local("text"));
                System.out.println("duplicate-false=" + InstanceOfProbe.local(Integer.valueOf(1)));
                break;
            case "stale":
                System.out.println("stale-true=" + InstanceOfProbe.local("text"));
                System.out.println("stale-false=" + InstanceOfProbe.local(Integer.valueOf(1)));
                break;
            case "retained":
                System.out.println("retained-true=" + InstanceOfProbe.local("text"));
                System.out.println("retained-false=" + InstanceOfProbe.local(Integer.valueOf(1)));
                break;
            case "int":
                System.out.println("int-string=" + InstanceOfProbe.branch("text"));
                System.out.println("int-number=" + InstanceOfProbe.branch(Integer.valueOf(1)));
                System.out.println("int-null=" + InstanceOfProbe.branch(null));
                break;
            default:
                throw new IllegalArgumentException(args[0]);
        }
    }

    private static void pop() {
        InstanceOfSupport.calls = 0;
        InstanceOfSupport.fail = false;
        try {
            // Reflection accepts both the original ()Z method and the patched ()V method.
            InstanceOfProbe.class.getMethod("called").invoke(null);
            System.out.println("pop=returned:" + InstanceOfSupport.calls);
        } catch (Throwable error) {
            Throwable actual = error;
            if (error instanceof InvocationTargetException) {
                actual = ((InvocationTargetException) error).getCause();
            }
            System.out.println(
                "pop=" + actual.getClass().getName() + ":" + InstanceOfSupport.calls
            );
        }
    }
}
