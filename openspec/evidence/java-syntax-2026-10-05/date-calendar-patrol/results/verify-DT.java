public class DT extends java.lang.Object {
    public DT() {
        super();
        return;
    }

    static java.lang.String fmt(java.util.Calendar arg0) {
        arg0.set(5, 1);
        arg0.add(2, 1);
        arg0.set(11, 0);
        return new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm").format((java.util.Date) arg0.getTime());
    }

    static java.util.Calendar parse(java.lang.String arg0) throws java.text.ParseException {
        java.util.Calendar local1 = java.util.Calendar.getInstance();
        local1.setTime((java.util.Date) new java.text.SimpleDateFormat("yyyy-MM-dd").parse(arg0));
        return local1;
    }

    static boolean after(java.lang.String arg0, java.lang.String arg1) throws java.text.ParseException {
        return parse(arg0).getTime().after((java.util.Date) parse(arg1).getTime());
    }

    static long daysBetween(java.util.Calendar arg0, java.util.Calendar arg1) {
        long local2 = arg1.getTimeInMillis() - arg0.getTimeInMillis();
        return local2 / 86400000L;
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        java.util.Calendar local1 = java.util.Calendar.getInstance();
        local1.set(2026, 8, 15, 10, 30);
        java.util.Calendar local2 = parse("2026-09-30");
        java.lang.System.out.println("" + fmt(local1) + "/" + daysBetween((java.util.Calendar) parse("2026-09-01"), local2) + "/" + after("2026-09-30", "2026-09-01"));
        return;
    }
}
