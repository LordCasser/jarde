package defpackage;

/* JADX INFO: loaded from: IO.class */
public class IO {
    static int countLines(java.lang.String str) throws java.io.IOException {
        java.io.BufferedReader bufferedReader = new java.io.BufferedReader(new java.io.InputStreamReader(new java.io.FileInputStream(str), "UTF-8"));
        int i = 0;
        while (bufferedReader.readLine() != null) {
            try {
                i++;
            } catch (java.lang.Throwable th) {
                bufferedReader.close();
                throw th;
            }
        }
        int i2 = i;
        bufferedReader.close();
        return i2;
    }

    static java.lang.String readAll(java.lang.String str) throws java.io.IOException {
        java.io.FileReader fileReader = new java.io.FileReader(str);
        java.lang.StringBuilder sb = new java.lang.StringBuilder();
        while (true) {
            try {
                int i = fileReader.read();
                if (i == -1) {
                    fileReader.close();
                    return sb.toString();
                }
                sb.append((char) i);
            } catch (java.lang.Throwable th) {
                fileReader.close();
                throw th;
            }
        }
    }

    public static void main(java.lang.String[] strArr) throws java.lang.Exception {
        java.lang.System.out.println("" + countLines("data.txt") + "/" + readAll("data.txt").replace("\n", "|"));
    }
}
