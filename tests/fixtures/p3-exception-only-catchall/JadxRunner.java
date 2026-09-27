package defpackage;

public class JadxRunner {
    public static void main(String[] args) {
        try {
            FinallyOnce.escaping();
            System.out.println("returned:" + FinallyOnce.count());
        } catch (Throwable error) {
            System.out.println(error.getClass().getName() + ":" + error.getMessage() + ":" + FinallyOnce.count());
        }
    }
}
