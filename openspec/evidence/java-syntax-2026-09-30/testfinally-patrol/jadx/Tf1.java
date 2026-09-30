package defpackage;

/* JADX INFO: loaded from: Tf1.class */
public class Tf1 {
    String test(Context context, Object obj) {
        Cursor cursor = null;
        try {
            Cursor cursorQuery = context.query(obj, new String[]{"name"});
            int columnIndexOrThrow = cursorQuery.getColumnIndexOrThrow("name");
            cursorQuery.moveToFirst();
            String string = cursorQuery.getString(columnIndexOrThrow);
            if (cursorQuery != null) {
            }
            return string;
        } finally {
            if (cursor != null) {
                cursor.close();
            }
        }
    }
}
