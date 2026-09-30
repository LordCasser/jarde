// Negative 5 (task 1.2, "copy grammar addition"): the guarded cleanup carries a second
// statement after the close, so each copy is eight instructions of guard plus call plus call —
// not the fixed four-instruction (or four-plus-tail) grammar the certificate proves.
public class Tf1CleanupExtra {
    static int closes = 0;

    String test(Context context, Object uri) {
        Cursor cursor = null;
        try {
            String[] projection = { "name" };
            cursor = context.query(uri, projection);
            int columnIndex = cursor.getColumnIndexOrThrow("name");
            cursor.moveToFirst();
            return cursor.getString(columnIndex);
        } finally {
            if (cursor != null) {
                cursor.close();
                Throwables.count("extra");
            }
        }
    }

    public static void main(String[] args) {
        Tf1CleanupExtra t = new Tf1CleanupExtra();
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
        Tf1CleanupExtra.closes++;
    }

    void moveToFirst() { }

    int getColumnIndexOrThrow(String s) {
        return 0;
    }

    String getString(int i) {
        return "v";
    }
}

class Throwables {
    static void count(String w) { }
}
