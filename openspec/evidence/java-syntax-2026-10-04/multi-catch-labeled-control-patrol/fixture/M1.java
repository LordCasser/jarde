import java.io.*;
import java.util.*;
public class M1 {
    // multi-catch (Java 7+)
    static int multi(String s) {
        try { return Integer.parseInt(s); }
        catch (NumberFormatException | NullPointerException e) { return -1; }
        catch (Exception e) { return -2; }
    }
    // labeled break/continue on nested loops
    static int labeled() {
        outer: for (int i = 0; i < 3; i++) {
            for (int j = 0; j < 3; j++) {
                if (j == 1) continue outer;
                if (i == 2) break outer;
            }
        }
        return 7;
    }
    // labeled break out of switch inside loop
    static String labeledSwitch(int n) {
        StringBuilder sb = new StringBuilder();
        loop: for (int i = 0; i < n; i++) {
            switch (i) {
                case 0: sb.append('a'); break;
                case 1: sb.append('b'); break loop;
                default: sb.append('c');
            }
        }
        return sb.toString();
    }
    public static void main(String[] a) {
        System.out.println(multi("42") + "/" + multi("x") + "/" + multi(null));
        System.out.println(labeled());
        System.out.println(labeledSwitch(3));
    }
}
