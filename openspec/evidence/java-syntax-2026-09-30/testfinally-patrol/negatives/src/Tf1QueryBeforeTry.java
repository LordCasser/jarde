// Negative 1b (task 1.2, "no lead null initialization"): the cleanup local is assigned before
// the try, so the protected row does not start at the statement's own block and the lead range
// is a full expression sequence, not the two-instruction `[aconst_null, astore s]`.
public class Tf1QueryBeforeTry {
    static int closes = 0;

    String test(Context context, Object uri) {
        Cursor cursor = context.query(uri, new String[] { "name" });
        try {
            int columnIndex = cursor.getColumnIndexOrThrow("name");
            cursor.moveToFirst();
            return cursor.getString(columnIndex);
        } finally {
            if (cursor != null) {
                cursor.close();
            }
        }
    }

    public static void main(String[] args) {
        Tf1QueryBeforeTry t = new Tf1QueryBeforeTry();
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
        Tf1QueryBeforeTry.closes++;
    }

    void moveToFirst() { }

    int getColumnIndexOrThrow(String s) {
        return 0;
    }

    String getString(int i) {
        return "v";
    }
}
