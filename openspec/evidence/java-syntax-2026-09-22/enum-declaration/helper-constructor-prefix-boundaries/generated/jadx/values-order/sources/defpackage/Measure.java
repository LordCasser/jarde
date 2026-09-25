package defpackage;

/* JADX INFO: loaded from: Measure.class */
enum Measure {
    HIGH(5),
    LOW(2);

    final int units;
    static int totalUnits = sumUnits();

    Measure(int i) {
        this.units = i;
    }

    static int sumUnits() {
        return LOW.units + HIGH.units;
    }
}
