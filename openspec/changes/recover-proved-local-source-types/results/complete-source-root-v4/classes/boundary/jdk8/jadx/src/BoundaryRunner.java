/** Stable input driver for full-class source/runtime comparisons. */
public final class BoundaryRunner {
    private BoundaryRunner() {
    }

    public static void main(String[] args) {
        System.out.println("char-call-dot=" + LocalSourceTypesBoundaries.charCallAndLiteralWrites(0, false, false));
        System.out.println("char-call-zero=" + LocalSourceTypesBoundaries.charCallAndLiteralWrites(2, true, false));
        System.out.println("char-call-max=" + LocalSourceTypesBoundaries.charCallAndLiteralWrites(0, false, true));
        System.out.println("char-field=" + LocalSourceTypesBoundaries.charFieldSeed());
        System.out.println("char-i2c-low=" + LocalSourceTypesBoundaries.charI2cSeed(-1));
        System.out.println("char-i2c-high=" + LocalSourceTypesBoundaries.charI2cSeed(65536));
        System.out.println("char-entry=" + LocalSourceTypesBoundaries.charEntryParameterSeed('?', false));
        System.out.println("char-entry-max=" + LocalSourceTypesBoundaries.charEntryParameterSeed('.', true));

        System.out.println("int-low=" + LocalSourceTypesBoundaries.intWithOutOfRangeWrites(0, 7));
        System.out.println("int-high=" + LocalSourceTypesBoundaries.intWithOutOfRangeWrites(1, 7));
        System.out.println("int-input=" + LocalSourceTypesBoundaries.intWithOutOfRangeWrites(2, 65536));
        System.out.println("int-arithmetic=" + LocalSourceTypesBoundaries.intWithArithmeticWrite(true));
        System.out.println("int-arithmetic-other=" + LocalSourceTypesBoundaries.intWithArithmeticWrite(false));
        System.out.println("int-copy=" + LocalSourceTypesBoundaries.intWithUnknownCopyMerge(true, 65536));
        System.out.println("int-merge-literal=" + LocalSourceTypesBoundaries.intWithUnknownCopyMerge(false, 9));

        System.out.print("string-no-default-one=");
        LocalSourceTypesBoundaries.exactStringWritesAfterNull(1);
        System.out.print("string-no-default-two=");
        LocalSourceTypesBoundaries.exactStringWritesAfterNull(2);
        System.out.print("string-no-default-miss=");
        LocalSourceTypesBoundaries.exactStringWritesAfterNull(9);

        System.out.print("reference-string=");
        LocalSourceTypesBoundaries.mixedReferenceWrites(true);
        System.out.print("reference-builder=");
        LocalSourceTypesBoundaries.mixedReferenceWrites(false);
        System.out.print("reference-all-null-true=");
        LocalSourceTypesBoundaries.allNullWrites(true);
        System.out.print("reference-all-null-false=");
        LocalSourceTypesBoundaries.allNullWrites(false);
        System.out.print("reference-opaque-null=");
        LocalSourceTypesBoundaries.unknownReferenceCopy(true, null);
        System.out.print("reference-opaque-string=");
        LocalSourceTypesBoundaries.unknownReferenceCopy(true, "input");
        System.out.print("reference-known-string=");
        LocalSourceTypesBoundaries.unknownReferenceCopy(false, "ignored");

        System.out.println("slot-reuse-int-string=" + LocalSourceTypesBoundaries.possibleSlotReuse(true));
        System.out.println("slot-reuse-int-copy=" + LocalSourceTypesBoundaries.possibleSlotReuse(false));
    }
}
