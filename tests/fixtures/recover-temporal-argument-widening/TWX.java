import java.time.*;
import java.time.temporal.ChronoUnit;
public class TWX {
    static String month(){ return java.time.format.DateTimeFormatter.ofPattern("MM-dd").format(MonthDay.of(1, 2)); }   // MonthDay 的 header 逐字声明 TemporalAccessor，但不在封闭十四行
    static long year(){ return ChronoUnit.DAYS.between(Year.of(2026), Year.of(2027)); }                                // Year 的 header 逐字声明 Temporal，但不在封闭十四行
}
