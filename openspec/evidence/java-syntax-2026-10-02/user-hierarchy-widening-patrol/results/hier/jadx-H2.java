
/* JADX INFO: loaded from: hier-neg.jar:H2.class */
public class H2 {

    /* JADX INFO: loaded from: hier-neg.jar:H2$Ext.class */
    static class Ext extends Helper {
        Ext() {
        }
    }

    /* JADX INFO: loaded from: hier-neg.jar:H2$MyErr.class */
    static class MyErr extends Exception {
        MyErr(String str) {
            super(str);
        }
    }

    /* JADX INFO: loaded from: hier-neg.jar:H2$Target.class */
    interface Target {
        String tag();
    }

    static String takeT(Target target) {
        return "t:" + target.tag();
    }

    static void sink(Throwable th) {
        System.out.println("sinkT:" + th.getMessage());
    }

    public static void main(String[] strArr) {
        System.out.println(takeT(new Ext()));
        sink(new MyErr("m"));
    }
}
