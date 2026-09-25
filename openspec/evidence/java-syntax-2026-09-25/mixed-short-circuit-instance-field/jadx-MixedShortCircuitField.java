
/* JADX WARN: Classes with same name are omitted, all sources:
  sample.jar:MixedShortCircuitField.class
  sample.jar:openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-instance-field/sample.jar:MixedShortCircuitField.class
 */
/* JADX INFO: loaded from: sample.jar:MixedShortCircuitField.class */
public final class MixedShortCircuitField {
    static boolean bValue;
    static boolean cValue;
    static int bCalls;
    static int cCalls;
    static int receiverCalls;
    static Box receiver;

    /* JADX WARN: Classes with same name are omitted, all sources:
      sample.jar:MixedShortCircuitField$Box.class
      sample.jar:openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-instance-field/sample.jar:MixedShortCircuitField$Box.class
     */
    /* JADX INFO: loaded from: sample.jar:MixedShortCircuitField$Box.class */
    static final class Box {
        boolean result;

        Box() {
        }
    }

    static Box target(boolean z) {
        receiverCalls++;
        if (z) {
            return null;
        }
        return receiver;
    }

    static void one(boolean z, boolean z2) {
        target(z2).result = (z && b()) || c();
    }

    static boolean b() {
        bCalls++;
        return bValue;
    }

    static boolean c() {
        cCalls++;
        return cValue;
    }
}
