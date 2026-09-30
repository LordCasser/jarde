// Negative 3a (task 1.2, "the two copies' slot disagrees with the body's assignment"): the
// lead's null and both cleanup copies name `spare`, while the body assigns only `cursor` — a
// different slot. The copies are still pairwise identical, but no body assignment of the
// cleanup slot exists, so the merged value flow the certificate must prove is absent.
public class Tf1SlotMismatch {
    static int closes = 0;

    String test(Context context, Object uri) {
        Cursor spare = null;
        try {
            Cursor cursor = context.query(uri, new String[] { "name" });
            int columnIndex = cursor.getColumnIndexOrThrow("name");
            cursor.moveToFirst();
            return cursor.getString(columnIndex);
        } finally {
            if (spare != null) {
                spare.close();
            }
        }
    }

    public static void main(String[] args) {
        Tf1SlotMismatch t = new Tf1SlotMismatch();
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
        Tf1SlotMismatch.closes++;
    }

    void moveToFirst() { }

    int getColumnIndexOrThrow(String s) {
        return 0;
    }

    String getString(int i) {
        return "v";
    }
}
