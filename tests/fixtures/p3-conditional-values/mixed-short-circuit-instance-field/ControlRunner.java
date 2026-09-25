public final class ControlRunner {
    public static void main(String[] args) {
        for (int mask = 0; mask < 8; mask++) {
            MixedInstanceControls.bValue = (mask & 2) != 0;
            MixedInstanceControls.cValue = (mask & 4) != 0;
            boolean a = (mask & 1) != 0;
            MixedInstanceControls.numeric(a, false);
            MixedInstanceControls.duplicated(a, false);
            MixedInstanceControls.compound(a, false);
            MixedInstanceControls.sharedReceiver(a, false);
            MixedInstanceControls.protectedWrite(a, false);
            MixedInstanceControls.inheritedOwner(a, false);
            if (MixedInstanceControls.lastDerived.result
                    != ((a && MixedInstanceControls.bValue) || MixedInstanceControls.cValue)) {
                throw new AssertionError("inherited field result for mask " + mask);
            }
        }
    }
}
