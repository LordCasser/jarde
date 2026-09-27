package dt31;
public final class DirectOrdinalRunner {
    public static void main(String[] args) {
        for (SparseSwitch.Count value : SparseSwitch.Count.values()) {
            System.out.println(value + "=" + SparseSwitch.select(value));
        }
    }
}
