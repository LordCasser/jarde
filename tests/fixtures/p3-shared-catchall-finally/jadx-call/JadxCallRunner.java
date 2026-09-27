package defpackage;

public class JadxCallRunner {
    public static void main(String[] args) {
        System.out.println(SharedFinallyCall.handled(false) + ":" + SharedFinallyCall.count());
        System.out.println(SharedFinallyCall.handled(true) + ":" + SharedFinallyCall.count());
    }
}
