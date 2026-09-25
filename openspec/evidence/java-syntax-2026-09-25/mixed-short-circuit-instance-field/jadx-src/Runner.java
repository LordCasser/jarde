
import java.io.PrintStream;

/* JADX WARN: Classes with same name are omitted, all sources:
  sample.jar:Runner.class
  sample.jar:openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-instance-field/sample.jar:Runner.class
 */
/* JADX INFO: loaded from: sample.jar:Runner.class */
public final class Runner {
    public static void main(String[] strArr) {
        int i = 0;
        while (i <= 1) {
            for (int i2 = 0; i2 < 8; i2++) {
                boolean z = (i2 & 1) != 0;
                boolean z2 = (i2 & 2) != 0;
                boolean z3 = (i2 & 4) != 0;
                boolean z4 = i != 0;
                MixedShortCircuitField.bValue = z2;
                MixedShortCircuitField.cValue = z3;
                MixedShortCircuitField.bCalls = 0;
                MixedShortCircuitField.cCalls = 0;
                MixedShortCircuitField.receiverCalls = 0;
                MixedShortCircuitField.receiver = new MixedShortCircuitField.Box();
                boolean z5 = false;
                try {
                    MixedShortCircuitField.one(z, z4);
                } catch (NullPointerException e) {
                    z5 = true;
                }
                boolean z6 = MixedShortCircuitField.receiver.result;
                PrintStream printStream = System.out;
                Object[] objArr = new Object[8];
                objArr[0] = Integer.valueOf(i);
                objArr[1] = Integer.valueOf(i2);
                objArr[2] = z5 ? "NPE" : "OK";
                objArr[3] = Boolean.valueOf(z6);
                objArr[4] = Integer.valueOf(MixedShortCircuitField.bCalls);
                objArr[5] = Integer.valueOf(MixedShortCircuitField.cCalls);
                objArr[6] = Integer.valueOf(MixedShortCircuitField.receiverCalls);
                objArr[7] = MixedShortCircuitField.receiver == null ? "null" : "box";
                printStream.printf("%d:%d:%s:%s:%d:%d:%d:%s%n", objArr);
            }
            i++;
        }
    }
}
