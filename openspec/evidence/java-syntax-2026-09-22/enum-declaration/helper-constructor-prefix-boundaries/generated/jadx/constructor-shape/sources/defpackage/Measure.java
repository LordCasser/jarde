package defpackage;

/* JADX INFO: loaded from: Measure.class */
enum Measure {
    LOW(2),
    HIGH(5);

    final int units;
    static int totalUnits = sumUnits();

    Measure(int i) {
        this.units = i;
        BoundarySupport.constructorEffect();
    }

    static int sumUnits() {
        return LOW.units + HIGH.units;
    }
}
