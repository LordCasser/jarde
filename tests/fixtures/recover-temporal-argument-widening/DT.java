import java.time.*;
import java.time.format.DateTimeFormatter;
public class DT {
    static String direct(){                                            // 直接参数（判别：非 aaload 来源）
        DateTimeFormatter f = DateTimeFormatter.ofPattern("HH:mm");
        LocalDateTime dt = LocalDateTime.of(2026, 1, 2, 3, 4);
        return f.format(dt);
    }
    static String parse(){                                             // parse 位（另一方向）
        return LocalTime.parse("03:04", DateTimeFormatter.ofPattern("HH:mm")).toString();
    }
    public static void main(String[] a){ System.out.println(direct()+"/"+parse()); }
}
