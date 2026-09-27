package p;

/* JADX INFO: loaded from: input.jar:p/Capture.class */
public class Capture {
    public static Runnable create(final double d) {
        return new Runnable() { // from class: p.Capture.1
            @Override // java.lang.Runnable
            public void run() {
                System.out.println(d);
            }
        };
    }
}
