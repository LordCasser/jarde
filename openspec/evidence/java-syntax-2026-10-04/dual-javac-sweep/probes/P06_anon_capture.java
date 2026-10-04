import java.util.*;
public class P06_anon_capture {
    static Runnable mk(final String s){ final int n = s.length(); return new Runnable(){ public void run(){ System.out.println(s+":"+n); } }; }
    public static void main(String[] a){ mk("abc").run(); }
}
