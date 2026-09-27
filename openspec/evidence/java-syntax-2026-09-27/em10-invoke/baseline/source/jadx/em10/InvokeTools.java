package em10;

/* JADX INFO: loaded from: fixture.jar:em10/InvokeTools.class */
class InvokeTools {
    InvokeTools() {
    }

    static long combine(int i, long j, double d, int i2) {
        return ((long) i) + j + ((long) d) + ((long) i2);
    }

    static String caught(String str) {
        return "caught:" + str;
    }
}
