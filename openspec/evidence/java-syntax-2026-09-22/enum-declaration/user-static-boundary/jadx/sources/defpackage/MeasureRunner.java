package defpackage;

/* JADX INFO: loaded from: MeasureRunner.class */
class MeasureRunner {
    MeasureRunner() {
    }

    static void check(boolean z, String str) {
        if (!z) {
            throw new AssertionError(str);
        }
    }

    public static void main(String[] strArr) {
        Measure[] measureArrValues = Measure.values();
        check(measureArrValues.length == 2, "values count");
        check(measureArrValues[0] == Measure.LOW && measureArrValues[1] == Measure.HIGH, "values order");
        check(Measure.valueOf("LOW") == Measure.LOW, "valueOf");
        check(Measure.LOW.units == 2 && Measure.HIGH.units == 5, "constructor values");
        check(Measure.totalUnits == 7 && Measure.sumUnits() == 7, "user static initialization/method");
        System.out.println("user static boundary: PASS");
    }
}
