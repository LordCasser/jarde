/** Source-only runner for the complete-class JVM comparison. */
public final class InstanceOfRunner {
    private InstanceOfRunner() {}

    public static void main(String[] args) {
        System.out.println("string-text=" + InstanceOfProbe.objectString("text"));
        System.out.println("string-null=" + InstanceOfProbe.objectString(null));
        System.out.println("string-number=" + InstanceOfProbe.objectString(Integer.valueOf(1)));
        System.out.println("null=" + InstanceOfProbe.nullValue());
        System.out.println("runnable=" + InstanceOfProbe.objectRunnable(new Runnable() {
            public void run() {}
        }));
        System.out.println("runnable-null=" + InstanceOfProbe.objectRunnable(null));
        System.out.println("primitive-array=" + InstanceOfProbe.primitiveArray(new int[0]));
        System.out.println("primitive-array-string=" + InstanceOfProbe.primitiveArray(new String[0]));
        System.out.println("reference-array=" + InstanceOfProbe.referenceArray(new String[0]));
        System.out.println("reference-array-object=" + InstanceOfProbe.referenceArray(new Object[0]));
        System.out.println("multi-array=" + InstanceOfProbe.multiArray(new String[0][0]));
        System.out.println("multi-array-one=" + InstanceOfProbe.multiArray(new String[0]));
        System.out.println("widened-string=" + InstanceOfProbe.widenedString("text"));
        System.out.println("local-true=" + InstanceOfProbe.local("text"));
        System.out.println("local-false=" + InstanceOfProbe.local(Integer.valueOf(1)));
        System.out.println("parameter-true=" + InstanceOfProbe.parameter("text"));
        System.out.println("parameter-false=" + InstanceOfProbe.parameter(Integer.valueOf(1)));
        System.out.println("branch-true=" + InstanceOfProbe.branch("text"));
        System.out.println("branch-false=" + InstanceOfProbe.branch(Integer.valueOf(1)));
        System.out.println("functional=" + InstanceOfProbe.functional());

        InstanceOfSupport.calls = 0;
        InstanceOfSupport.fail = false;
        System.out.println("called=" + InstanceOfProbe.called() + ":" + InstanceOfSupport.calls);
        InstanceOfSupport.calls = 0;
        InstanceOfSupport.fail = true;
        try {
            InstanceOfProbe.called();
            System.out.println("called-fail=returned:" + InstanceOfSupport.calls);
        } catch (RuntimeException error) {
            System.out.println("called-fail=" + error.getClass().getName() + ":" + InstanceOfSupport.calls);
        }
    }
}
