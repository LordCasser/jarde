import java.time.*;
import java.time.format.DateTimeFormatter;
public class JT {
    static String basic(){                                              // LocalDate/LocalDateTime 工厂+链
        LocalDate d = LocalDate.of(2026, 10, 6);
        LocalDate later = d.plusDays(10).plusMonths(1).withDayOfMonth(1);
        LocalDateTime dt = later.atTime(9, 30);
        return d + "/" + later + "/" + dt.toLocalTime();
    }
    static String fmt(){                                                // DateTimeFormatter 双向
        DateTimeFormatter f = DateTimeFormatter.ofPattern("yyyy/MM/dd HH:mm");
        LocalDateTime dt = LocalDateTime.of(2026, 1, 2, 3, 4);
        LocalDateTime back = LocalDateTime.parse(f.format(dt), f);
        return f.format(dt) + " -> " + back.getHour() + ":" + back.getMinute();
    }
    static long spans(){                                                // Duration/Period/ChronoUnit
        LocalDate a = LocalDate.of(2026, 1, 1);
        LocalDate b = LocalDate.of(2026, 3, 15);
        Duration dur = Duration.between(a.atStartOfDay(), b.atStartOfDay());
        Period per = Period.between(a, b);
        return dur.toDays() + java.time.temporal.ChronoUnit.MONTHS.between(a, b) * 100 + per.getDays();
    }
    public static void main(String[] x){
        System.out.println(basic());
        System.out.println(fmt());
        System.out.println(spans());
    }
}
