class Context { Cursor query(Object o, String[] s) { return new Cursor(); } }
class Cursor {
    void close() { Throwables.count("close"); }
    void moveToFirst() { }
    int getColumnIndexOrThrow(String s) { return 0; }
    String getString(int i) { return "v"; }
}
class Throwables { static int closes = 0; static void count(String w) { closes++; } }
public class Tf1 {
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
}
