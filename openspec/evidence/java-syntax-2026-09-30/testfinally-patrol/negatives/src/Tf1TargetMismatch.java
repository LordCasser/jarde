// Patch base for the bytecode-patched negative `Tf1TargetMismatch`: the fixed Tf1 shape itself, compiled
// fresh so `negatives/patch.py` can patch its copy bytes in place. The patch this class feeds
// is described in `patch.py`; the source is deliberately the exact fixed lowering.
public class Tf1TargetMismatch {
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
            }
        }
    }

    public static void main(String[] args) {
        Tf1TargetMismatch t = new Tf1TargetMismatch();
        String outcome = "ok";
        String value = null;
        try {
            value = t.test(new Context(), new Object());
        } catch (Throwable thrown) {
            outcome = thrown.getClass().getSimpleName() + ":" + thrown.getMessage();
        }
        System.out.println("outcome=" + outcome + " value=" + value + " closes=" + closes);
    }
}

class Context {
    Cursor query(Object o, String[] s) {
        return new Cursor();
    }
}

class Cursor {
    void close() {
        Tf1TargetMismatch.closes++;
    }

    void moveToFirst() { }

    int getColumnIndexOrThrow(String s) {
        return 0;
    }

    String getString(int i) {
        return "v";
    }
}
