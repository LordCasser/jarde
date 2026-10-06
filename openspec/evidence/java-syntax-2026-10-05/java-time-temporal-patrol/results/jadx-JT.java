package defpackage;

/* JADX INFO: loaded from: JT.class */
public class JT {
    static java.lang.String basic() {
        java.time.LocalDate localDateOf = java.time.LocalDate.of(2026, 10, 6);
        java.time.LocalDate localDateWithDayOfMonth = localDateOf.plusDays(10L).plusMonths(1L).withDayOfMonth(1);
        return localDateOf + "/" + localDateWithDayOfMonth + "/" + localDateWithDayOfMonth.atTime(9, 30).toLocalTime();
    }

    static java.lang.String fmt() {
        java.time.format.DateTimeFormatter dateTimeFormatterOfPattern = java.time.format.DateTimeFormatter.ofPattern("yyyy/MM/dd HH:mm");
        java.time.LocalDateTime localDateTimeOf = java.time.LocalDateTime.of(2026, 1, 2, 3, 4);
        java.time.LocalDateTime localDateTime = java.time.LocalDateTime.parse(dateTimeFormatterOfPattern.format(localDateTimeOf), dateTimeFormatterOfPattern);
        return dateTimeFormatterOfPattern.format(localDateTimeOf) + " -> " + localDateTime.getHour() + ":" + localDateTime.getMinute();
    }

    static long spans() {
        java.time.LocalDate localDateOf = java.time.LocalDate.of(2026, 1, 1);
        java.time.LocalDate localDateOf2 = java.time.LocalDate.of(2026, 3, 15);
        return java.time.Duration.between(localDateOf.atStartOfDay(), localDateOf2.atStartOfDay()).toDays() + (java.time.temporal.ChronoUnit.MONTHS.between(localDateOf, localDateOf2) * 100) + ((long) java.time.Period.between(localDateOf, localDateOf2).getDays());
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(basic());
        java.lang.System.out.println(fmt());
        java.lang.System.out.println(spans());
    }
}
