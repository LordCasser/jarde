// Negative 1a (task 1.2, "no lead null initialization"): the cleanup local's lead stores a
// fresh `Cursor`, not the certificate's `[aconst_null, astore s]`. The cleanup is still a
// conditional close, but the null source the certificate must pin to the lead is absent, so
// the fixed local-null lowering is not what the bytes say.
public class Tf1LeadNew {
    static int closes = 0;

    String test(Context context, Object uri) {
        Cursor cursor = new Cursor();
        try {
            String[] projection = { "name" };
            cursor = context.query(uri, projection);
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
        Tf1LeadNew t = new Tf1LeadNew();
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
        Tf1LeadNew.closes++;
    }

    void moveToFirst() { }

    int getColumnIndexOrThrow(String s) {
        return 0;
    }

    String getString(int i) {
        return "v";
    }
}
