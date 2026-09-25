package defpackage;

/* JADX INFO: loaded from: BoundaryRunner.class */
class BoundaryRunner {
    BoundaryRunner() {
    }

    static String lookup(String str) {
        try {
            return Measure.valueOf(str).name();
        } catch (Throwable th) {
            return th.getClass().getSimpleName();
        }
    }

    public static void main(String[] strArr) {
        Measure[] measureArrValues = Measure.values();
        StringBuilder sb = new StringBuilder();
        for (Measure measure : measureArrValues) {
            if (sb.length() != 0) {
                sb.append(',');
            }
            sb.append(measure.name()).append(':').append(measure.ordinal()).append(':').append(measure.units);
        }
        System.out.println("values=" + ((Object) sb));
        System.out.println("lookup=LOW:" + lookup("LOW") + ",HIGH:" + lookup("HIGH") + ",MISSING:" + lookup("MISSING"));
        System.out.println("sum=" + Measure.totalUnits + ":" + Measure.sumUnits());
        System.out.println("helper-calls=" + BoundarySupport.calls);
    }
}
