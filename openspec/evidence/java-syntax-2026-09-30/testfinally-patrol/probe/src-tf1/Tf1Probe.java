// The same-layout probe whose body can fail after the assignment and whose cleanup can itself
// fail: the certificate's parameterized guarded-throw tail. The three flags are static so the
// reflection runner can drive every path against each recompiled side.
public class Tf1Probe {
    public static boolean failCleanup = false;

    public String test(Context context, Object uri) {
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
                if (failCleanup) {
                    throw new RuntimeException("cleanup");
                }
            }
        }
    }
}

class Context {
    public static boolean failQuery = false;

    Cursor query(Object o, String[] s) {
        if (Context.failQuery) {
            return null;
        }
        return new Cursor();
    }
}

class Cursor {
    public static boolean failBody = false;

    void close() {
        Throwables.count("close");
    }

    void moveToFirst() {
        if (Cursor.failBody) {
            throw new RuntimeException("body");
        }
    }

    int getColumnIndexOrThrow(String s) {
        return 0;
    }

    String getString(int i) {
        return "v";
    }
}

class Throwables {
    public static int closes = 0;

    static void count(String w) {
        closes++;
    }
}
