package defpackage;

/* JADX INFO: loaded from: Measure.class */
enum Measure {
    LOW(BoundarySupport.prefixValue(2)),
    HIGH(BoundarySupport.prefixValue(5));

    final int units;
    static int totalUnits = sumUnits();

    Measure(int i) {
        this.units = i;
    }

    static int sumUnits() {
        return LOW.units + HIGH.units;
    }
}
