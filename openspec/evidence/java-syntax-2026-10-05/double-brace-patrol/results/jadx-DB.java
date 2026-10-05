package defpackage;

/* JADX INFO: loaded from: DB.class */
public class DB {
    static java.util.List<java.lang.String> dbl = new java.util.ArrayList<java.lang.String>() { // from class: DB.1
        {
            add("a");
            add("b");
        }
    };

    static int size() {
        return dbl.size();
    }

    static java.util.List<java.lang.String> withCapture(final java.lang.String str) {
        return new java.util.ArrayList<java.lang.String>() { // from class: DB.2
            {
                add(str);
            }
        };
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + size() + "/" + withCapture("z").get(0));
    }
}
