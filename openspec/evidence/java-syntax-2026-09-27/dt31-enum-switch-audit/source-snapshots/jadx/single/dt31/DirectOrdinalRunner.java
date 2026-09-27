package dt31;

/* JADX INFO: loaded from: inputs.jar:dt31/DirectOrdinalRunner.class */
public final class DirectOrdinalRunner {
    public static void main(String[] strArr) {
        for (SparseSwitch.Count count : SparseSwitch.Count.values()) {
            System.out.println(count + "=" + SparseSwitch.select(count));
        }
    }
}
