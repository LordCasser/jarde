package defpackage;

/* JADX INFO: loaded from: Tf1Probe.class */
public class Tf1Probe {
    public static boolean failCleanup = false;

    public String test(Context context, Object obj) {
        Cursor cursorQuery = null;
        try {
            cursorQuery = context.query(obj, new String[]{"name"});
            int columnIndexOrThrow = cursorQuery.getColumnIndexOrThrow("name");
            cursorQuery.moveToFirst();
            String string = cursorQuery.getString(columnIndexOrThrow);
            if (cursorQuery != null) {
                cursorQuery.close();
                if (failCleanup) {
                    throw new RuntimeException("cleanup");
                }
            }
            return string;
        } catch (Throwable th) {
            if (cursorQuery != null) {
                cursorQuery.close();
                if (failCleanup) {
                    throw new RuntimeException("cleanup");
                }
            }
            throw th;
        }
    }
}
