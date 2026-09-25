package defpackage;

final class ProbeRunner {
    static String run(String method, String fail) {
        IterableExceptionProbe.log = "";
        IterableExceptionProbe.caught = 0;
        IterableExceptionProbe.finalized = 0;
        Iterable values = new IterableExceptionProbe.Source(fail);
        int result;
        try {
            if ("uniform".equals(method)) result = IterableExceptionProbe.uniform(values);
            else if ("iteratorBoundary".equals(method)) result = IterableExceptionProbe.iteratorBoundary(values);
            else result = IterableExceptionProbe.nextBoundary(values);
        } catch (RuntimeException ex) {
            result = -99;
            IterableExceptionProbe.log += "X";
        }
        return method + "/" + fail + "=" + result + ";" + IterableExceptionProbe.log
                + ";caught=" + IterableExceptionProbe.caught + ";finally=" + IterableExceptionProbe.finalized;
    }
    public static void main(String[] args) {
        String[] methods = { "uniform", "iteratorBoundary", "nextBoundary" };
        String[] failures = { "none", "iterator", "hasNext", "next" };
        for (String method : methods) for (String fail : failures) System.out.println(run(method, fail));
    }
}
