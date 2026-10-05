import java.util.*;
import java.text.*;
public class DT {
    static String fmt(Calendar c){                                                 // Calendar 字段读写+格式化
        c.set(Calendar.DAY_OF_MONTH, 1);
        c.add(Calendar.MONTH, 1);
        c.set(Calendar.HOUR_OF_DAY, 0);
        return new SimpleDateFormat("yyyy-MM-dd HH:mm").format(c.getTime());
    }
    static Calendar parse(String s) throws ParseException {
        Calendar c = Calendar.getInstance();
        c.setTime(new SimpleDateFormat("yyyy-MM-dd").parse(s));
        return c;
    }
    static boolean after(String a, String b) throws ParseException {              // Date 比较
        return parse(a).getTime().after(parse(b).getTime());
    }
    static long daysBetween(Calendar from, Calendar to){                          // 日差计算惯用法
        long ms = to.getTimeInMillis() - from.getTimeInMillis();
        return ms / (24 * 60 * 60 * 1000);
    }
    public static void main(String[] x) throws Exception {
        Calendar c = Calendar.getInstance(); c.set(2026, Calendar.SEPTEMBER, 15, 10, 30);
        Calendar d = parse("2026-09-30");
        System.out.println(""+fmt(c)+"/"+daysBetween(parse("2026-09-01"), d)+"/"+after("2026-09-30","2026-09-01"));
    }
}
