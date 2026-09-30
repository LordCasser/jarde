// Negative 4 (task 1.2, "same-slot assignment outside the protected range"): the body never
// assigns `cursor`; the assignment the cleanup's null test would read sits after the finally
// statement. A same-slot store outside the body range is exactly what the certificate's store
// census refuses, and the statement's saved-return completion is gone with it.
public class Tf1AssignOutside {
    static int closes = 0;

    String test(Context context, Object uri) {
        Cursor cursor = null;
        try {
            uri.toString();
        } finally {
            if (cursor != null) {
                cursor.close();
            }
        }
        cursor = context.query(uri, new String[] { "name" });
        cursor.moveToFirst();
        return "done";
    }

    public static void main(String[] args) {
        Tf1AssignOutside t = new Tf1AssignOutside();
        String value = t.test(new Context(), new Object());
        System.out.println("value=" + value + " closes=" + closes);
    }
}

class Context {
    Cursor query(Object o, String[] s) {
        return new Cursor();
    }
}

class Cursor {
    void close() {
        Tf1AssignOutside.closes++;
    }

    void moveToFirst() { }

    int getColumnIndexOrThrow(String s) {
        return 0;
    }

    String getString(int i) {
        return "v";
    }
}
